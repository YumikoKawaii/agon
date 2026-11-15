package rpchandler

import (
	"context"

	"connectrpc.com/connect"
	pb "github.com/yumikokawaii/agon/gen/agon/v1"
	"github.com/yumikokawaii/agon/gen/agon/v1/agonv1connect"
	svc "github.com/yumikokawaii/agon/internal/service"
)

type AuthServiceHandler struct {
	svc *svc.AuthService
}

var _ agonv1connect.AuthServiceHandler = (*AuthServiceHandler)(nil)

func NewAuthServiceHandler(s *svc.AuthService) *AuthServiceHandler {
	return &AuthServiceHandler{svc: s}
}

func (h *AuthServiceHandler) Register(ctx context.Context, req *connect.Request[pb.RegisterRequest]) (*connect.Response[pb.RegisterResponse], error) {
	u, token, err := h.svc.Register(req.Msg.Username, req.Msg.Password)
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.RegisterResponse{Token: token, User: userToProto(u)}), nil
}

func (h *AuthServiceHandler) Login(ctx context.Context, req *connect.Request[pb.LoginRequest]) (*connect.Response[pb.LoginResponse], error) {
	u, token, err := h.svc.Login(req.Msg.Username, req.Msg.Password)
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.LoginResponse{Token: token, User: userToProto(u)}), nil
}

func (h *AuthServiceHandler) Logout(ctx context.Context, req *connect.Request[pb.LogoutRequest]) (*connect.Response[pb.LogoutResponse], error) {
	return connect.NewResponse(&pb.LogoutResponse{}), nil
}
