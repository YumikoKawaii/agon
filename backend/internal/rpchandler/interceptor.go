package rpchandler

import (
	"context"
	"net/http"
	"strings"

	"connectrpc.com/connect"
	"github.com/yumikokawaii/agon/gen/agon/v1/agonv1connect"
	svc "github.com/yumikokawaii/agon/internal/service"
)

type Authenticator struct {
	auth *svc.AuthService
}

var _ connect.Interceptor = (*Authenticator)(nil)

func NewAuthenticator(auth *svc.AuthService) *Authenticator {
	return &Authenticator{auth: auth}
}

func public(procedure string) bool {
	switch procedure {
	case agonv1connect.AuthServiceRegisterProcedure, agonv1connect.AuthServiceLoginProcedure:
		return true
	}
	return false
}

func (a *Authenticator) authenticate(ctx context.Context, procedure string, h http.Header) (context.Context, error) {
	if public(procedure) {
		return ctx, nil
	}
	token, found := strings.CutPrefix(h.Get("Authorization"), "Bearer ")
	if !found || token == "" {
		return nil, connect.NewError(connect.CodeUnauthenticated, errNotSignedIn)
	}
	claims, err := a.auth.ParseJWT(token)
	if err != nil {
		return nil, connect.NewError(connect.CodeUnauthenticated, errNotSignedIn)
	}
	return withUser(ctx, claims.UserID), nil
}

func (a *Authenticator) WrapUnary(next connect.UnaryFunc) connect.UnaryFunc {
	return func(ctx context.Context, req connect.AnyRequest) (connect.AnyResponse, error) {
		ctx, err := a.authenticate(ctx, req.Spec().Procedure, req.Header())
		if err != nil {
			return nil, err
		}
		return next(ctx, req)
	}
}

func (a *Authenticator) WrapStreamingClient(next connect.StreamingClientFunc) connect.StreamingClientFunc {
	return next
}

func (a *Authenticator) WrapStreamingHandler(next connect.StreamingHandlerFunc) connect.StreamingHandlerFunc {
	return func(ctx context.Context, conn connect.StreamingHandlerConn) error {
		ctx, err := a.authenticate(ctx, conn.Spec().Procedure, conn.RequestHeader())
		if err != nil {
			return err
		}
		return next(ctx, conn)
	}
}
