"use strict";
const agon_reqs = {};
const agon_watches = {};
const agon_wts = {};
let agon_next = 1;
function agon_str(p, l) {
    return new TextDecoder().decode(new Uint8Array(wasm_memory.buffer, p, l));
}
function agon_put(p, bytes) {
    new Uint8Array(wasm_memory.buffer, p, bytes.length).set(bytes);
}
function agon_headers(t, type) {
    const h = { "Content-Type": type, "Connect-Protocol-Version": "1" };
    if (t) h.Authorization = "Bearer " + t;
    return h;
}
function agon_hex(s) {
    const b = new Uint8Array(s.length / 2);
    for (let i = 0; i < b.length; i++) b[i] = parseInt(s.substr(i * 2, 2), 16);
    return b;
}
async function agon_read_watch(w, res) {
    const reader = res.body.getReader();
    let buf = new Uint8Array();
    for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        const merged = new Uint8Array(buf.length + value.length);
        merged.set(buf);
        merged.set(value, buf.length);
        buf = merged;
        while (buf.length >= 5) {
            const flags = buf[0];
            const len = new DataView(buf.buffer, buf.byteOffset + 1, 4).getUint32(0);
            if (buf.length < 5 + len) break;
            const msg = buf.slice(5, 5 + len);
            buf = buf.slice(5 + len);
            if (flags & 2) {
                const end = JSON.parse(new TextDecoder().decode(msg) || "{}");
                if (end.error) w.error = new TextEncoder().encode(end.error.message || end.error.code || "stream error");
            } else {
                w.queue.push(msg);
            }
        }
    }
}
async function agon_read_lines(c, stream) {
    const reader = stream.getReader();
    let text = "";
    for (;;) {
        const { done, value } = await reader.read();
        if (done) break;
        text += new TextDecoder().decode(value);
        let i;
        while ((i = text.indexOf("\n")) >= 0) {
            c.lines.push(new TextEncoder().encode(text.slice(0, i)));
            text = text.slice(i + 1);
        }
    }
}
miniquad_add_plugin({
    name: "agon_net",
    version: 2,
    register_plugin: function (o) {
        o.env.agon_fetch = function (mp, ml, up, ul, bp, bl, tp, tl) {
            const id = agon_next++;
            const r = (agon_reqs[id] = { done: false, status: 0, body: new Uint8Array() });
            const opt = { method: agon_str(mp, ml), headers: agon_headers(agon_str(tp, tl), "application/json") };
            opt.body = bl > 0 ? agon_str(bp, bl) : "{}";
            fetch(agon_str(up, ul), opt)
                .then(async (x) => {
                    r.status = x.status;
                    r.body = new Uint8Array(await x.arrayBuffer());
                    r.done = true;
                })
                .catch(() => {
                    r.status = -1;
                    r.done = true;
                });
            return id;
        };
        o.env.agon_status = function (id) {
            const r = agon_reqs[id];
            return r && r.done ? r.status : 0;
        };
        o.env.agon_len = function (id) {
            return agon_reqs[id].body.length;
        };
        o.env.agon_copy = function (id, p) {
            agon_put(p, agon_reqs[id].body);
            delete agon_reqs[id];
        };
        o.env.agon_drop = function (id) {
            delete agon_reqs[id];
        };
        o.env.agon_watch = function (up, ul, bp, bl, tp, tl) {
            const id = agon_next++;
            const w = (agon_watches[id] = { queue: [], closed: false, error: null, ctl: new AbortController() });
            const json = new TextEncoder().encode(bl > 0 ? agon_str(bp, bl) : "{}");
            const body = new Uint8Array(5 + json.length);
            new DataView(body.buffer).setUint32(1, json.length);
            body.set(json, 5);
            fetch(agon_str(up, ul), { method: "POST", headers: agon_headers(agon_str(tp, tl), "application/connect+json"), body, signal: w.ctl.signal })
                .then(async (res) => {
                    if (!res.ok) {
                        w.error = new TextEncoder().encode("error " + res.status);
                        return;
                    }
                    await agon_read_watch(w, res);
                })
                .catch(() => {})
                .finally(() => {
                    w.closed = true;
                });
            return id;
        };
        o.env.agon_watch_next = function (id) {
            const w = agon_watches[id];
            if (!w) return -2;
            if (w.queue.length) return w.queue[0].length;
            return w.closed ? -2 : -1;
        };
        o.env.agon_watch_copy = function (id, p) {
            agon_put(p, agon_watches[id].queue.shift());
        };
        o.env.agon_watch_close = function (id) {
            const w = agon_watches[id];
            if (w) w.ctl.abort();
            delete agon_watches[id];
        };
        o.env.agon_wt_open = function (up, ul, hp, hl) {
            const id = agon_next++;
            const c = (agon_wts[id] = { state: 0, dgrams: [], lines: [], writer: null, wt: null });
            try {
                const wt = new WebTransport(agon_str(up, ul), {
                    serverCertificateHashes: [{ algorithm: "sha-256", value: agon_hex(agon_str(hp, hl)) }],
                });
                c.wt = wt;
                wt.ready
                    .then(async () => {
                        c.state = 1;
                        c.writer = wt.datagrams.writable.getWriter();
                        (async () => {
                            const r = wt.datagrams.readable.getReader();
                            for (;;) {
                                const { done, value } = await r.read();
                                if (done) break;
                                c.dgrams.push(value);
                                if (c.dgrams.length > 64) c.dgrams.shift();
                            }
                        })().catch(() => {});
                        const uni = wt.incomingUnidirectionalStreams.getReader();
                        for (;;) {
                            const { done, value } = await uni.read();
                            if (done) break;
                            agon_read_lines(c, value).catch(() => {});
                        }
                    })
                    .catch(() => {});
                wt.closed.catch(() => {}).finally(() => {
                    c.state = 2;
                });
            } catch (e) {
                c.state = 2;
            }
            return id;
        };
        o.env.agon_wt_state = function (id) {
            const c = agon_wts[id];
            return c ? c.state : 2;
        };
        o.env.agon_wt_send = function (id, p, l) {
            const c = agon_wts[id];
            if (c && c.writer) c.writer.write(new Uint8Array(wasm_memory.buffer, p, l).slice()).catch(() => {});
        };
        o.env.agon_wt_recv = function (id, p, max) {
            const c = agon_wts[id];
            if (!c || !c.dgrams.length) return 0;
            const d = c.dgrams.shift();
            if (d.length > max) return 0;
            agon_put(p, d);
            return d.length;
        };
        o.env.agon_wt_line = function (id) {
            const c = agon_wts[id];
            return c && c.lines.length ? c.lines[0].length : -1;
        };
        o.env.agon_wt_line_copy = function (id, p) {
            agon_put(p, agon_wts[id].lines.shift());
        };
        o.env.agon_wt_close = function (id) {
            const c = agon_wts[id];
            if (c && c.wt) {
                try { c.wt.close(); } catch (e) {}
            }
            delete agon_wts[id];
        };
    },
    on_init: function () {},
});
