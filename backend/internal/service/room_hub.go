package service

import "sync"

type RoomHub struct {
	mu   sync.Mutex
	subs map[string]map[chan struct{}]struct{}
}

func NewRoomHub() *RoomHub {
	return &RoomHub{subs: map[string]map[chan struct{}]struct{}{}}
}

func (h *RoomHub) Subscribe(id string) (<-chan struct{}, func()) {
	c := make(chan struct{}, 1)
	h.mu.Lock()
	if h.subs[id] == nil {
		h.subs[id] = map[chan struct{}]struct{}{}
	}
	h.subs[id][c] = struct{}{}
	h.mu.Unlock()
	return c, func() {
		h.mu.Lock()
		delete(h.subs[id], c)
		if len(h.subs[id]) == 0 {
			delete(h.subs, id)
		}
		h.mu.Unlock()
	}
}

func (h *RoomHub) Notify(id string) {
	h.mu.Lock()
	defer h.mu.Unlock()
	for c := range h.subs[id] {
		select {
		case c <- struct{}{}:
		default:
		}
	}
}
