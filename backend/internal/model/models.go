package model

import "time"

type RoomState int32

const (
	RoomWaiting RoomState = 1
	RoomPlaying RoomState = 2
	RoomClosed  RoomState = 3
)

type User struct {
	ID           int64
	Name         string
	PasswordHash string
	Kit          int32
	CreatedAt    time.Time
	UpdatedAt    time.Time
}

type Room struct {
	ID         string
	HostID     int64
	GuestID    *int64
	HostReady  bool
	GuestReady bool
	State      RoomState
	MatchID    string
	CreatedAt  time.Time
	UpdatedAt  time.Time
	Host       User  `gorm:"foreignKey:HostID"`
	Guest      *User `gorm:"foreignKey:GuestID"`
}

func (r *Room) Seat(userID int64) (int, bool) {
	switch {
	case r.HostID == userID:
		return 0, true
	case r.GuestID != nil && *r.GuestID == userID:
		return 1, true
	}
	return 0, false
}

type Match struct {
	ID          string
	RoomID      string
	HostID      int64
	GuestID     int64
	WinnerID    *int64
	HostDeaths  int32
	GuestDeaths int32
	StartedAt   time.Time `gorm:"autoCreateTime"`
	EndedAt     *time.Time
	Winner      *User `gorm:"foreignKey:WinnerID"`
}

func (m *Match) Player(seat int) int64 {
	if seat == 0 {
		return m.HostID
	}
	return m.GuestID
}

func (m *Match) Has(userID int64) bool {
	return m.HostID == userID || m.GuestID == userID
}
