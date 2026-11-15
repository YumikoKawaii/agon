package arena

import (
	"crypto/rand"
	"crypto/sha256"
	"crypto/tls"
	"encoding/binary"
	"encoding/hex"
	"log"
	"net"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/quic-go/quic-go"
	"github.com/quic-go/quic-go/http3"
	"github.com/quic-go/webtransport-go"
)

const (
	PacketLen   = 44
	deathsAt    = 36
	DeathsToWin = 3
	closeDelay  = 2 * time.Second
)

type Result struct {
	MatchID    string
	WinnerSeat int
	Deaths     [2]int32
}

type ticket struct {
	match string
	seat  int
}

type match struct {
	sess   [2]*webtransport.Session
	ctrl   [2]*webtransport.SendStream
	deaths [2]uint32
	done   bool
}

type Arena struct {
	wt      *webtransport.Server
	url     string
	hash    string
	results chan Result

	mu      sync.Mutex
	matches map[string]*match
	tickets map[string]ticket
}

func New(port, host string) (*Arena, error) {
	cert, err := selfSigned(host)
	if err != nil {
		return nil, err
	}
	sum := sha256.Sum256(cert.Leaf.Raw)
	a := &Arena{
		url:     "https://" + net.JoinHostPort(host, port) + "/match",
		hash:    hex.EncodeToString(sum[:]),
		results: make(chan Result, 16),
		matches: map[string]*match{},
		tickets: map[string]ticket{},
	}
	h3 := &http3.Server{
		Addr:      ":" + port,
		TLSConfig: http3.ConfigureTLSConfig(&tls.Config{Certificates: []tls.Certificate{cert}}),
		QUICConfig: &quic.Config{
			EnableDatagrams:                  true,
			EnableStreamResetPartialDelivery: true,
			KeepAlivePeriod:                  5 * time.Second,
		},
	}
	webtransport.ConfigureHTTP3Server(h3)
	mux := http.NewServeMux()
	mux.HandleFunc("/match", a.handle)
	h3.Handler = mux
	a.wt = &webtransport.Server{H3: h3, CheckOrigin: func(*http.Request) bool { return true }}
	return a, nil
}

func (a *Arena) Serve() error { return a.wt.ListenAndServe() }

func (a *Arena) Results() <-chan Result { return a.results }

func (a *Arena) Address() (string, string) { return a.url, a.hash }

func (a *Arena) Open(matchID string) {
	a.mu.Lock()
	defer a.mu.Unlock()
	a.matches[matchID] = &match{}
}

func (a *Arena) Ticket(matchID string, seat int) (string, bool) {
	a.mu.Lock()
	defer a.mu.Unlock()
	if m := a.matches[matchID]; m == nil || m.done {
		return "", false
	}
	b := make([]byte, 16)
	rand.Read(b)
	t := hex.EncodeToString(b)
	a.tickets[t] = ticket{matchID, seat}
	return t, true
}

func (a *Arena) Forfeit(matchID string, loserSeat int) {
	a.finish(matchID, 1-loserSeat)
}

func (a *Arena) handle(w http.ResponseWriter, r *http.Request) {
	key := r.URL.Query().Get("ticket")
	a.mu.Lock()
	t, ok := a.tickets[key]
	delete(a.tickets, key)
	m := a.matches[t.match]
	a.mu.Unlock()
	if !ok || m == nil {
		w.WriteHeader(http.StatusForbidden)
		return
	}
	sess, err := a.wt.Upgrade(w, r)
	if err != nil {
		log.Printf("arena: upgrade: %v", err)
		return
	}
	ctrl, err := sess.OpenUniStreamSync(r.Context())
	if err != nil {
		sess.CloseWithError(1, "control stream")
		return
	}
	if _, err := ctrl.Write([]byte("seat " + strconv.Itoa(t.seat) + "\n")); err != nil {
		sess.CloseWithError(1, "control stream")
		return
	}
	a.mu.Lock()
	if old := m.sess[t.seat]; old != nil {
		old.CloseWithError(2, "replaced")
	}
	m.sess[t.seat] = sess
	m.ctrl[t.seat] = ctrl
	if m.sess[0] != nil && m.sess[1] != nil && !m.done {
		broadcast(m, "start")
	}
	a.mu.Unlock()
	go a.pump(t.match, m, t.seat, sess)
}

func broadcast(m *match, line string) {
	for _, c := range m.ctrl {
		if c != nil {
			c.Write([]byte(line + "\n"))
		}
	}
}

func (a *Arena) pump(id string, m *match, seat int, sess *webtransport.Session) {
	ctx := sess.Context()
	for {
		b, err := sess.ReceiveDatagram(ctx)
		if err != nil {
			break
		}
		if len(b) != PacketLen {
			continue
		}
		a.mu.Lock()
		if m.sess[seat] != sess || m.done {
			a.mu.Unlock()
			continue
		}
		m.deaths[seat] = max(m.deaths[seat], binary.LittleEndian.Uint32(b[deathsAt:]))
		other := m.sess[1-seat]
		lost := m.deaths[seat] >= DeathsToWin
		a.mu.Unlock()
		if other != nil {
			other.SendDatagram(b)
		}
		if lost {
			a.finish(id, 1-seat)
		}
	}
	a.mu.Lock()
	gone := m.sess[seat] == sess && !m.done
	a.mu.Unlock()
	if gone {
		a.finish(id, 1-seat)
	}
}

func (a *Arena) finish(id string, winnerSeat int) {
	a.mu.Lock()
	m := a.matches[id]
	if m == nil || m.done {
		a.mu.Unlock()
		return
	}
	m.done = true
	broadcast(m, "end "+strconv.Itoa(winnerSeat))
	sess := m.sess
	deaths := [2]int32{int32(m.deaths[0]), int32(m.deaths[1])}
	a.mu.Unlock()
	a.results <- Result{MatchID: id, WinnerSeat: winnerSeat, Deaths: deaths}
	time.AfterFunc(closeDelay, func() {
		for _, s := range sess {
			if s != nil {
				s.CloseWithError(0, "match over")
			}
		}
		a.mu.Lock()
		delete(a.matches, id)
		a.mu.Unlock()
	})
}
