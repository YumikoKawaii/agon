package rpchandler

import (
	"context"

	"connectrpc.com/connect"
	pb "github.com/yumikokawaii/agon/gen/agon/v1"
	"github.com/yumikokawaii/agon/gen/agon/v1/agonv1connect"
	"github.com/yumikokawaii/agon/internal/model"
	svc "github.com/yumikokawaii/agon/internal/service"
)

type RoomServiceHandler struct {
	svc *svc.RoomService
}

var _ agonv1connect.RoomServiceHandler = (*RoomServiceHandler)(nil)

func NewRoomServiceHandler(s *svc.RoomService) *RoomServiceHandler {
	return &RoomServiceHandler{svc: s}
}

func (h *RoomServiceHandler) CreateRoom(ctx context.Context, req *connect.Request[pb.CreateRoomRequest]) (*connect.Response[pb.CreateRoomResponse], error) {
	r, err := h.svc.Create(userFrom(ctx))
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.CreateRoomResponse{Room: roomToProto(r)}), nil
}

func (h *RoomServiceHandler) JoinRoom(ctx context.Context, req *connect.Request[pb.JoinRoomRequest]) (*connect.Response[pb.JoinRoomResponse], error) {
	r, err := h.svc.Join(userFrom(ctx), req.Msg.RoomId)
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.JoinRoomResponse{Room: roomToProto(r)}), nil
}

func (h *RoomServiceHandler) LeaveRoom(ctx context.Context, req *connect.Request[pb.LeaveRoomRequest]) (*connect.Response[pb.LeaveRoomResponse], error) {
	if err := h.svc.Leave(userFrom(ctx), req.Msg.RoomId); err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.LeaveRoomResponse{}), nil
}

func (h *RoomServiceHandler) GetRoom(ctx context.Context, req *connect.Request[pb.GetRoomRequest]) (*connect.Response[pb.GetRoomResponse], error) {
	r, err := h.svc.Get(userFrom(ctx), req.Msg.RoomId)
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.GetRoomResponse{Room: roomToProto(r)}), nil
}

func (h *RoomServiceHandler) WatchRoom(ctx context.Context, req *connect.Request[pb.WatchRoomRequest], stream *connect.ServerStream[pb.WatchRoomResponse]) error {
	err := h.svc.Watch(ctx, userFrom(ctx), req.Msg.RoomId, func(r *model.Room) error {
		return stream.Send(&pb.WatchRoomResponse{Room: roomToProto(r)})
	})
	return toConnectError(err)
}

func (h *RoomServiceHandler) SetReady(ctx context.Context, req *connect.Request[pb.SetReadyRequest]) (*connect.Response[pb.SetReadyResponse], error) {
	r, err := h.svc.SetReady(userFrom(ctx), req.Msg.RoomId, req.Msg.Ready)
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.SetReadyResponse{Room: roomToProto(r)}), nil
}
