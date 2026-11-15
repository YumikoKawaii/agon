package rpchandler

import (
	pb "github.com/yumikokawaii/agon/gen/agon/v1"
	"github.com/yumikokawaii/agon/internal/model"
)

func userToProto(u *model.User) *pb.User {
	return &pb.User{Username: u.Name, Kit: u.Kit}
}

func roomToProto(r *model.Room) *pb.Room {
	room := &pb.Room{
		Id:      r.ID,
		Host:    &pb.Seat{Username: r.Host.Name, Kit: r.Host.Kit, Ready: r.HostReady},
		Guest:   &pb.Seat{},
		State:   pb.RoomState(r.State),
		MatchId: r.MatchID,
	}
	if r.Guest != nil {
		room.Guest = &pb.Seat{Username: r.Guest.Name, Kit: r.Guest.Kit, Ready: r.GuestReady}
	}
	return room
}
