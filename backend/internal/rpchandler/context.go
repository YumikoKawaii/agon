package rpchandler

import "context"

type userKey struct{}

func withUser(ctx context.Context, id int64) context.Context {
	return context.WithValue(ctx, userKey{}, id)
}

func userFrom(ctx context.Context) int64 {
	id, _ := ctx.Value(userKey{}).(int64)
	return id
}
