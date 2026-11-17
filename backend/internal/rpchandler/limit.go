package rpchandler

import (
	"errors"
	"net"
	"sync"
	"time"
)

const (
	authWindow   = time.Minute
	authAttempts = 10
)

var errTooMany = errors.New("too many attempts, try again in a minute")

type window struct {
	count int
	reset time.Time
}

type limiter struct {
	mu   sync.Mutex
	hits map[string]*window
}

func newLimiter() *limiter {
	return &limiter{hits: map[string]*window{}}
}

func (l *limiter) allow(addr string) bool {
	host, _, err := net.SplitHostPort(addr)
	if err != nil {
		host = addr
	}
	now := time.Now()
	l.mu.Lock()
	defer l.mu.Unlock()
	for k, w := range l.hits {
		if now.After(w.reset) {
			delete(l.hits, k)
		}
	}
	w := l.hits[host]
	if w == nil {
		w = &window{reset: now.Add(authWindow)}
		l.hits[host] = w
	}
	w.count++
	return w.count <= authAttempts
}
