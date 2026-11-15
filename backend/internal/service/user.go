package service

import (
	"errors"

	"github.com/yumikokawaii/agon/internal/model"
	"github.com/yumikokawaii/agon/internal/repository"
)

const Kits = 3

type UserService struct {
	users repository.UserRepository
}

func NewUserService(users repository.UserRepository) *UserService {
	return &UserService{users: users}
}

func (s *UserService) Get(id int64) (*model.User, error) {
	u, err := s.users.FindByID(id)
	if errors.Is(err, repository.ErrNotFound) {
		return nil, fail(ErrUnauthenticated, "account no longer exists")
	}
	return u, err
}

func (s *UserService) SetKit(id int64, kit int32) (*model.User, error) {
	if kit < 0 || kit >= Kits {
		return nil, fail(ErrBadRequest, "bad kit")
	}
	if err := s.users.UpdateKit(id, kit); err != nil {
		return nil, err
	}
	return s.Get(id)
}
