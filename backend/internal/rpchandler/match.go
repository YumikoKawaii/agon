package rpchandler

import (
	"context"

	"connectrpc.com/connect"
	pb "github.com/yumikokawaii/agon/gen/agon/v1"
	"github.com/yumikokawaii/agon/gen/agon/v1/agonv1connect"
	svc "github.com/yumikokawaii/agon/internal/service"
)

type MatchServiceHandler struct {
	svc *svc.MatchService
}

var _ agonv1connect.MatchServiceHandler = (*MatchServiceHandler)(nil)

func NewMatchServiceHandler(s *svc.MatchService) *MatchServiceHandler {
	return &MatchServiceHandler{svc: s}
}

func (h *MatchServiceHandler) GetEndpoint(ctx context.Context, req *connect.Request[pb.GetEndpointRequest]) (*connect.Response[pb.GetEndpointResponse], error) {
	e, err := h.svc.Endpoint(userFrom(ctx), req.Msg.MatchId)
	if err != nil {
		return nil, toConnectError(err)
	}
	return connect.NewResponse(&pb.GetEndpointResponse{Url: e.URL, CertHash: e.CertHash, Ticket: e.Ticket, Seat: int32(e.Seat)}), nil
}

func (h *MatchServiceHandler) GetResult(ctx context.Context, req *connect.Request[pb.GetResultRequest]) (*connect.Response[pb.GetResultResponse], error) {
	m, err := h.svc.Result(userFrom(ctx), req.Msg.MatchId)
	if err != nil {
		return nil, toConnectError(err)
	}
	res := &pb.GetResultResponse{MatchId: m.ID, HostDeaths: m.HostDeaths, GuestDeaths: m.GuestDeaths, Finished: m.EndedAt != nil}
	if m.Winner != nil {
		res.Winner = m.Winner.Name
	}
	return connect.NewResponse(res), nil
}
