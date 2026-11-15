package repository

import (
	"errors"

	"gorm.io/gorm"
)

var (
	ErrNotFound = errors.New("record not found")
	ErrConflict = errors.New("already exists")
)

func translate(err error) error {
	switch {
	case errors.Is(err, gorm.ErrRecordNotFound):
		return ErrNotFound
	case errors.Is(err, gorm.ErrDuplicatedKey):
		return ErrConflict
	}
	return err
}
