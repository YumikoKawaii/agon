package service

import "errors"

var (
	ErrBadRequest      = errors.New("bad request")
	ErrUnauthenticated = errors.New("unauthenticated")
	ErrForbidden       = errors.New("forbidden")
	ErrConflict        = errors.New("conflict")
	ErrNotFound        = errors.New("not found")
	ErrPrecondition    = errors.New("failed precondition")
	ErrFull            = errors.New("full")
)

type Error struct {
	Kind error
	Msg  string
}

func (e *Error) Error() string { return e.Msg }
func (e *Error) Unwrap() error { return e.Kind }

func fail(kind error, msg string) error { return &Error{Kind: kind, Msg: msg} }
