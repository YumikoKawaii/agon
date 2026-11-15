package rpchandler

import (
	"errors"

	"connectrpc.com/connect"
	"github.com/yumikokawaii/agon/internal/service"
)

var errNotSignedIn = errors.New("not signed in")

func toConnectError(err error) error {
	if err == nil {
		return nil
	}
	switch {
	case errors.Is(err, service.ErrBadRequest):
		return connect.NewError(connect.CodeInvalidArgument, err)
	case errors.Is(err, service.ErrUnauthenticated):
		return connect.NewError(connect.CodeUnauthenticated, err)
	case errors.Is(err, service.ErrForbidden):
		return connect.NewError(connect.CodePermissionDenied, err)
	case errors.Is(err, service.ErrConflict):
		return connect.NewError(connect.CodeAlreadyExists, err)
	case errors.Is(err, service.ErrNotFound):
		return connect.NewError(connect.CodeNotFound, err)
	case errors.Is(err, service.ErrPrecondition):
		return connect.NewError(connect.CodeFailedPrecondition, err)
	case errors.Is(err, service.ErrFull):
		return connect.NewError(connect.CodeResourceExhausted, err)
	default:
		return connect.NewError(connect.CodeInternal, errors.New("server error"))
	}
}
