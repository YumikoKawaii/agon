package service

import (
	"context"
	"errors"

	"github.com/yumikokawaii/agon/internal/model"
	"github.com/yumikokawaii/agon/internal/repository"
	"github.com/yumikokawaii/agon/internal/uow"
)

type Endpoint struct {
	URL      string
	CertHash string
	Ticket   string
	Seat     int
}

type TicketIssuer interface {
	Ticket(matchID string, seat int) (string, bool)
	Address() (url, certHash string)
}

type MatchService struct {
	uow   uow.UnitOfWork
	hub   *RoomHub
	arena TicketIssuer
}

func NewMatchService(u uow.UnitOfWork, hub *RoomHub, arena TicketIssuer) *MatchService {
	return &MatchService{uow: u, hub: hub, arena: arena}
}

func (s *MatchService) member(userID int64, id string) (*model.Match, error) {
	m, err := s.uow.Store().Matches.FindByID(id)
	if errors.Is(err, repository.ErrNotFound) {
		return nil, fail(ErrNotFound, "match not found")
	}
	if err != nil {
		return nil, err
	}
	if !m.Has(userID) {
		return nil, fail(ErrForbidden, "not in this match")
	}
	return m, nil
}

func (s *MatchService) Endpoint(userID int64, id string) (*Endpoint, error) {
	m, err := s.member(userID, id)
	if err != nil {
		return nil, err
	}
	seat := 0
	if m.GuestID == userID {
		seat = 1
	}
	ticket, ok := s.arena.Ticket(id, seat)
	if !ok {
		return nil, fail(ErrPrecondition, "match not running")
	}
	url, hash := s.arena.Address()
	return &Endpoint{URL: url, CertHash: hash, Ticket: ticket, Seat: seat}, nil
}

func (s *MatchService) Result(userID int64, id string) (*model.Match, error) {
	return s.member(userID, id)
}

func (s *MatchService) AbortStale() error {
	return s.uow.Do(context.Background(), func(tx *uow.Store) error {
		if err := tx.Matches.AbortOpen(); err != nil {
			return err
		}
		return tx.Rooms.ResetPlaying()
	})
}

func (s *MatchService) Finish(id string, winnerSeat int, deaths [2]int32) error {
	var roomID string
	err := s.uow.Do(context.Background(), func(tx *uow.Store) error {
		m, err := tx.Matches.FindByID(id)
		if err != nil {
			return err
		}
		roomID = m.RoomID
		var winner *int64
		if winnerSeat == 0 || winnerSeat == 1 {
			p := m.Player(winnerSeat)
			winner = &p
		}
		if err := tx.Matches.Finish(id, winner, deaths); err != nil {
			return err
		}
		return tx.Rooms.EndMatch(m.RoomID, id)
	})
	if err != nil {
		return err
	}
	s.hub.Notify(roomID)
	return nil
}
