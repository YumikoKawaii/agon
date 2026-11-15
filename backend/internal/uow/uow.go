package uow

import (
	"context"

	"github.com/yumikokawaii/agon/internal/repository"
	"gorm.io/gorm"
)

type Store struct {
	Users   repository.UserRepository
	Rooms   repository.RoomRepository
	Matches repository.MatchRepository
}

func newStore(db *gorm.DB) *Store {
	return &Store{
		Users:   repository.NewUserRepo(db),
		Rooms:   repository.NewRoomRepo(db),
		Matches: repository.NewMatchRepo(db),
	}
}

type UnitOfWork interface {
	Store() *Store
	Do(ctx context.Context, fn func(s *Store) error) error
}

type unitOfWork struct {
	db    *gorm.DB
	store *Store
}

func New(db *gorm.DB) UnitOfWork {
	return &unitOfWork{db: db, store: newStore(db)}
}

func (u *unitOfWork) Store() *Store { return u.store }

func (u *unitOfWork) Do(ctx context.Context, fn func(s *Store) error) error {
	return u.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		return fn(newStore(tx))
	})
}
