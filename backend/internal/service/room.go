package service

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"time"

	"github.com/yumikokawaii/agon/internal/model"
	"github.com/yumikokawaii/agon/internal/repository"
	"github.com/yumikokawaii/agon/internal/uow"
)

const closedRoomTTL = time.Hour

type MatchHost interface {
	Open(matchID string)
	Forfeit(matchID string, loserSeat int)
}

type RoomService struct {
	uow   uow.UnitOfWork
	hub   *RoomHub
	arena MatchHost
}

func NewRoomService(u uow.UnitOfWork, hub *RoomHub, arena MatchHost) *RoomService {
	return &RoomService{uow: u, hub: hub, arena: arena}
}

func (s *RoomService) find(id string) (*model.Room, error) {
	r, err := s.uow.Store().Rooms.FindByID(id)
	if errors.Is(err, repository.ErrNotFound) {
		return nil, fail(ErrNotFound, "room not found")
	}
	return r, err
}

func (s *RoomService) member(userID int64, id string) (*model.Room, int, error) {
	r, err := s.find(id)
	if err != nil {
		return nil, 0, err
	}
	seat, ok := r.Seat(userID)
	if !ok {
		return nil, 0, fail(ErrForbidden, "not in this room")
	}
	return r, seat, nil
}

func (s *RoomService) leaveOthers(userID int64, keep string) error {
	ids, err := s.uow.Store().Rooms.ActiveFor(userID)
	if err != nil {
		return err
	}
	for _, id := range ids {
		if id == keep {
			continue
		}
		if err := s.Leave(userID, id); err != nil && !errors.Is(err, ErrForbidden) && !errors.Is(err, ErrNotFound) {
			return err
		}
	}
	return nil
}

func (s *RoomService) Sweep() error {
	return s.uow.Store().Rooms.DeleteClosedBefore(time.Now().Add(-closedRoomTTL))
}

func (s *RoomService) Create(userID int64) (*model.Room, error) {
	if err := s.leaveOthers(userID, ""); err != nil {
		return nil, err
	}
	for {
		n, err := rand.Int(rand.Reader, big.NewInt(1000000))
		if err != nil {
			return nil, err
		}
		id := fmt.Sprintf("%06d", n.Int64())
		err = s.uow.Store().Rooms.Create(&model.Room{ID: id, HostID: userID, State: model.RoomWaiting})
		if errors.Is(err, repository.ErrConflict) {
			continue
		}
		if err != nil {
			return nil, err
		}
		return s.find(id)
	}
}

func (s *RoomService) Join(userID int64, id string) (*model.Room, error) {
	if err := s.leaveOthers(userID, id); err != nil {
		return nil, err
	}
	if err := s.uow.Store().Rooms.Join(id, userID); err != nil {
		return nil, err
	}
	r, err := s.find(id)
	if err != nil {
		return nil, err
	}
	if r.State == model.RoomClosed {
		return nil, fail(ErrNotFound, "room closed")
	}
	if _, ok := r.Seat(userID); !ok {
		return nil, fail(ErrFull, "room is full")
	}
	s.hub.Notify(id)
	return r, nil
}

func (s *RoomService) Leave(userID int64, id string) error {
	r, seat, err := s.member(userID, id)
	if err != nil {
		return err
	}
	if r.State == model.RoomPlaying {
		s.arena.Forfeit(r.MatchID, seat)
	}
	rooms := s.uow.Store().Rooms
	if seat == 0 {
		err = rooms.Close(id)
	} else {
		err = rooms.RemoveGuest(id)
	}
	if err != nil {
		return err
	}
	s.hub.Notify(id)
	return nil
}

func (s *RoomService) Get(userID int64, id string) (*model.Room, error) {
	r, _, err := s.member(userID, id)
	return r, err
}

func (s *RoomService) Watch(ctx context.Context, userID int64, id string, send func(*model.Room) error) error {
	changed, stop := s.hub.Subscribe(id)
	defer stop()
	for {
		r, _, err := s.member(userID, id)
		if err != nil {
			return err
		}
		if err := send(r); err != nil {
			return err
		}
		if r.State == model.RoomClosed {
			return nil
		}
		select {
		case <-ctx.Done():
			return nil
		case <-changed:
		}
	}
}

func (s *RoomService) SetReady(userID int64, id string, ready bool) (*model.Room, error) {
	r, seat, err := s.member(userID, id)
	if err != nil {
		return nil, err
	}
	if r.State != model.RoomWaiting {
		return nil, fail(ErrPrecondition, "room is not waiting")
	}
	b := make([]byte, 8)
	rand.Read(b)
	matchID := hex.EncodeToString(b)
	var started *model.Match
	err = s.uow.Do(context.Background(), func(tx *uow.Store) error {
		if err := tx.Rooms.SetReady(id, seat, ready); err != nil {
			return err
		}
		ok, err := tx.Rooms.StartIfReady(id, matchID)
		if err != nil || !ok {
			return err
		}
		cur, err := tx.Rooms.FindByID(id)
		if err != nil {
			return err
		}
		started = &model.Match{ID: matchID, RoomID: id, HostID: cur.HostID, GuestID: *cur.GuestID}
		return tx.Matches.Create(started)
	})
	if err != nil {
		return nil, err
	}
	if started != nil {
		s.arena.Open(started.ID)
	}
	s.hub.Notify(id)
	return s.find(id)
}
