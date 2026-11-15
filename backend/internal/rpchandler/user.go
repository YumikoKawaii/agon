package rpchandler

import (
	"context"

	"connectrpc.com/connect"
	pb "github.com/yumikokawaii/agon/gen/agon/v1"
	"github.com/yumikokawaii/agon/gen/agon/v1/agonv1connect"
	svc "github.com/yumikokawaii/agon/internal/service"
)

type UserServiceHandler struct {
	svc *svc.UserService
}

var _ agonv1connect.UserServiceHandler = (*UserServiceHandler)(nil)

func NewUserServiceHandler(s *svc.UserService) *UserServiceHandler {
	return &UserServiceHandler{svc: s}
}

func (h *UserServiceHandler) GetMe(ctx context.Context, req *connect.Request[pb.GetMeRequest]) (*connect.Response[pb.GetMeResponse], error) {
	u, err := h.svc.Get(userFrom(ctx))
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.GetMeResponse{User: userToProto(u)}), nil
}

func (h *UserServiceHandler) SetKit(ctx context.Context, req *connect.Request[pb.SetKitRequest]) (*connect.Response[pb.SetKitResponse], error) {
	u, err := h.svc.SetKit(userFrom(ctx), req.Msg.Kit)
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.SetKitResponse{User: userToProto(u)}), nil
}
