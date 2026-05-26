use std::io;

use crate::rng::XorShift;

pub enum WordList {
    Dev,
    Casual,
    Music,
    Chatting,
}

impl WordList {
    pub fn words(&self) -> &'static [&'static str] {
        match self {
            WordList::Dev => DEV,
            WordList::Casual => CASUAL,
            WordList::Music => MUSIC,
            WordList::Chatting => CHATTING,
        }
    }
}

impl std::str::FromStr for WordList {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dev" => Ok(WordList::Dev),
            "casual" => Ok(WordList::Casual),
            "music" => Ok(WordList::Music),
            "chatting" => Ok(WordList::Chatting),
            _ => Err(()),
        }
    }
}

pub fn get_target_word_list(list: WordList) -> io::Result<String> {
    let tw = get_terminal_width()?;
    let target_width = tw.saturating_sub(5);
    if target_width == 0 {
        return Err(io::Error::other("terminal too narrow"));
    }

    let mut buff = String::with_capacity(target_width);
    let mut rng = XorShift::new();

    let words = list.words();
    if words.is_empty() {
        return Err(io::Error::other("word list is empty"));
    }

    let mut attempts = 0;
    while buff.len() < target_width {
        let next_word = words[rng.next_bound(words.len())];
        if buff.len() + next_word.len() < target_width {
            buff.push_str(next_word);
            buff.push(' ');
            attempts = 0;
        } else {
            attempts += 1;
            if attempts >= words.len() {
                break;
            }
        }
    }
    if buff.ends_with(' ') {
        buff.pop();
    }

    Ok(buff)
}

fn get_terminal_width() -> io::Result<usize> {
    winsize_cols(STDOUT_FILENO).or_else(|_| {
        // stdout may be piped; fall back to /dev/tty
        let fd = open_dev_tty()?;
        let result = winsize_cols(fd);
        unsafe { libc_close(fd) };
        result
    })
}

fn winsize_cols(fd: i32) -> io::Result<usize> {
    // `struct winsize` layout: ws_row, ws_col, ws_xpixel, ws_ypixel (all u16)
    let mut ws: [u16; 4] = [0; 4];
    let ret = unsafe {
        libc_ioctl(fd, TIOCGWINSZ, &mut ws as *mut [u16; 4] as *mut std::ffi::c_void)
    };
    if ret == -1 {
        return Err(io::Error::last_os_error());
    }
    let cols = ws[1] as usize;
    if cols == 0 {
        return Err(io::Error::other("terminal reported zero columns"));
    }
    Ok(cols)
}

fn open_dev_tty() -> io::Result<i32> {
    let path = b"/dev/tty\0";
    let fd =
        unsafe { libc_open(path.as_ptr() as *const std::ffi::c_char, 0 /* O_RDONLY */) };
    if fd == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}

#[cfg(target_os = "linux")]
const TIOCGWINSZ: u64 = 0x5413;
#[cfg(target_os = "macos")]
const TIOCGWINSZ: u64 = 0x40087468;

unsafe extern "C" {
    #[link_name = "ioctl"]
    fn libc_ioctl(fd: i32, request: u64, ...) -> i32;
    #[link_name = "open"]
    fn libc_open(path: *const std::ffi::c_char, flags: i32, ...) -> i32;
    #[link_name = "close"]
    fn libc_close(fd: i32) -> i32;
}

const STDOUT_FILENO: i32 = 1;

static CASUAL: &[&str] = &[
    "hello", "thanks", "please", "sorry", "ok", "okay", "yes", "no", "maybe", "sure", "cool",
    "great", "good", "bad", "fine", "nice", "awesome", "amazing", "funny", "crazy", "weird",
    "strange", "interesting", "bored", "tired", "sleep", "hungry", "eat", "drink", "coffee",
    "tea", "water", "morning", "evening", "night", "day", "today", "tomorrow", "yesterday",
    "week", "month", "year", "soon", "later", "always", "never", "sometimes", "often",
    "usually", "right", "left", "up", "down", "here", "there", "where", "when", "how", "what",
    "who", "which", "why", "because", "and", "or", "but", "so", "if", "then", "also", "more",
    "less", "little", "big", "small", "long", "short", "fast", "slow", "hot", "cold", "warm",
    "happy", "sad", "angry", "excited", "scared", "love", "like", "hate", "know", "think",
    "feel", "see", "look", "watch", "hear", "listen", "talk", "say", "ask", "tell", "read",
    "write", "play", "game", "music", "movie", "book", "story", "walk", "run", "drive",
    "travel", "home", "work", "school", "friend", "family", "people", "person", "child", "man",
    "woman", "boy", "girl", "dog", "cat", "animal", "nature", "sun", "moon", "star", "rain",
    "snow", "wind", "cloud", "sky", "tree", "flower", "car", "bike", "phone", "computer",
    "laptop", "internet", "online", "offline",
    // extended
    "remember", "forget", "wonder", "happen", "change", "believe", "suppose", "notice",
    "decide", "imagine", "realize", "explain", "describe", "suggest", "promise", "accept",
    "enjoy", "prefer", "continue", "begin", "seem", "appear", "become", "different", "same",
    "important", "possible", "difficult", "special", "beautiful", "wonderful", "terrible",
    "perfect", "simple", "usual", "normal", "common", "moment", "place", "reason", "answer",
    "question", "problem", "idea", "money", "time", "door", "window", "room", "house", "city",
    "country", "street", "market", "office", "hospital", "garden", "kitchen", "dinner",
    "breakfast", "lunch", "bread", "fruit", "vegetable", "chair", "table", "bag", "shirt",
    "shoes", "watch", "clock", "lamp", "mirror", "picture", "letter", "gift", "holiday",
    "summer", "winter", "spring", "autumn", "weather", "river", "ocean", "mountain", "island",
    "bridge", "airport", "station", "ticket", "journey", "adventure", "memory", "dream",
    "surprise", "secret", "choice", "habit", "effort", "result", "success", "failure",
    "beginning", "ending", "middle", "everything", "nothing", "something", "anyone", "nobody",
];

static DEV: &[&str] = &[
    "fix", "add", "remove", "change", "refactor", "merge", "branch", "commit", "push", "pull",
    "review", "test", "build", "run", "error", "debug", "log", "warn", "info", "start", "stop",
    "deploy", "release", "version", "config", "setup", "install", "check", "create", "delete",
    "init", "done", "work", "task", "issue", "bug", "feature", "improve", "optimize", "clean",
    "move", "replace", "copy", "generate", "message", "note", "comment", "fail", "success",
    "retry", "upgrade", "downgrade", "patch", "hotfix", "rollback", "plan", "schedule",
    "reviewed", "approved", "rejected", "prototype", "experiment", "design", "code", "script",
    "automation", "manual", "execute", "compile", "performance", "benchmark", "profile",
    "analyze", "monitor", "trace", "inspect", "validate", "verify", "confirm", "conflict",
    "resolve", "staging", "production", "backup", "restore", "sync", "tag", "changelog",
    "documentation", "docs", "wiki", "guide", "tutorial", "example", "snippet", "reference",
    "shortcut", "command", "terminal", "console", "shell", "exception", "handle", "abort",
    "crash", "panic", "queue", "job", "process", "thread", "lock", "mutex", "race", "condition",
    "deadlock", "timeout", "threshold", "limit", "capacity", "scale", "load", "stress",
    "latency", "throughput", "availability", "uptime", "downtime", "maintenance", "notify",
    "alert", "email", "chat", "discussion", "team", "collaboration", "meeting", "retrospective",
    "standup", "sprint", "milestone", "deadline", "priority", "high", "low", "medium",
    "blocked", "ready", "closed", "open", "assign", "owner", "report", "ticket", "incident",
    "audit", "security", "access", "permission", "role", "policy", "compliance",
    // extended
    "abstract", "interface", "struct", "enum", "trait", "closure", "lambda", "callback",
    "async", "await", "future", "promise", "channel", "socket", "buffer", "stream", "parser",
    "lexer", "token", "syntax", "runtime", "heap", "stack", "pointer", "reference", "borrow",
    "lifetime", "generic", "macro", "module", "package", "crate", "namespace", "import",
    "export", "library", "framework", "dependency", "injection", "pattern", "singleton",
    "factory", "observer", "adapter", "proxy", "cache", "index", "query", "schema", "migrate",
    "seed", "fixture", "mock", "stub", "coverage", "assertion", "regression", "canary",
    "rollout", "feature", "flag", "toggle", "webhook", "endpoint", "payload", "header",
    "token", "cookie", "session", "hash", "encrypt", "decrypt", "sign", "verify", "cert",
    "pipeline", "workflow", "trigger", "artifact", "container", "image", "volume", "cluster",
    "node", "replica", "shard", "partition", "index", "replication", "consensus", "quorum",
    "leader", "follower", "snapshot", "journal", "diff", "patch", "rebase", "cherry",
    "squash", "stash", "blame", "bisect", "hook", "lint", "format", "typecheck",
];

static MUSIC: &[&str] = &[
    "note", "scale", "chord", "melody", "harmony", "rhythm", "tempo", "beat", "song", "tune",
    "lyrics", "verse", "chorus", "bridge", "refrain", "instrument", "guitar", "piano", "drums",
    "bass", "violin", "flute", "trumpet", "saxophone", "clarinet", "cello", "keyboard", "organ",
    "synthesizer", "microphone", "vocal", "sing", "play", "listen", "compose", "arrange",
    "perform", "record", "mix", "master", "studio", "band", "orchestra", "ensemble", "solo",
    "duet", "trio", "quartet", "concert", "gig", "festival", "album", "track", "single",
    "release", "hit", "chart", "cover", "remix", "acoustic", "electric", "classical", "jazz",
    "blues", "rock", "pop", "hiphop", "rap", "metal", "punk", "folk", "reggae", "country",
    "samba", "bossa", "opera", "ballad", "anthem", "overture", "symphony", "sonata", "prelude",
    "fugue", "improvise", "jam", "riff", "lick", "groove", "hook", "beatbox", "percussion",
    "tambourine", "cymbal", "triangle", "maracas", "castanets", "harp", "mandolin", "banjo",
    "ukulele", "accordion",
    // extended
    "pitch", "interval", "octave", "major", "minor", "sharp", "flat", "natural", "rest",
    "bar", "measure", "downbeat", "upbeat", "swing", "syncopation", "dynamics", "forte",
    "piano", "mezzo", "crescendo", "decrescendo", "staccato", "legato", "vibrato", "tremolo",
    "arpeggio", "glissando", "pizzicato", "sustain", "reverb", "delay", "distortion",
    "equalizer", "compressor", "limiter", "sample", "loop", "quantize", "transpose", "pitch",
    "waveform", "frequency", "amplitude", "timbre", "overtone", "resonance", "modulation",
    "arpeggiator", "sequencer", "oscillator", "envelope", "filter", "vocoder", "theremin",
    "sitar", "djembe", "congas", "bongos", "xylophone", "vibraphone", "marimba", "trombone",
    "tuba", "oboe", "bassoon", "harpsichord", "clavinet", "mellotron", "sampler",
    "turntable", "vinyl", "cassette", "streaming", "playlist", "setlist", "encore",
    "interlude", "coda", "reprise", "medley", "mashup", "bootleg", "demo", "acoustic",
];

static CHATTING: &[&str] = &[
    "hi", "hey", "hello", "hiya", "yo", "sup", "morning", "evening", "night", "afternoon",
    "good", "great", "ok", "okay", "cool", "nice", "awesome", "fun", "lol", "lmao", "rofl",
    "haha", "hehe", "omg", "wtf", "brb", "btw", "idk", "imo", "imho", "fyi", "tbh", "np",
    "ty", "thx", "pls", "please", "sorry", "jk", "omw", "ttyl", "cya", "bbl", "gtg", "afk",
    "smh", "ikr", "irl", "rn", "fomo", "yolo", "bae", "fam", "bro", "sis", "dude", "mate",
    "friend", "buddy", "love", "like", "hate", "ugh", "meh", "wow", "oops", "yay", "hmm",
    "huh", "yep", "nope", "maybe", "sure", "okie", "deal", "thanks", "cheers", "welcome",
    "goodbye", "bye", "see", "later", "soon", "tonight", "today", "tomorrow", "yesterday",
    "weekend", "party", "chat", "talk", "message", "text", "call", "video", "meet", "online",
    "offline", "friendship", "team", "support", "help", "funny", "crazy", "weird",
    "interesting", "bored", "tired", "sleep", "hungry", "eat", "drink", "coffee", "tea",
    "water", "food", "cake", "pizza", "movie", "music", "game", "watch", "book", "story",
    "travel", "home", "work", "school", "study", "class", "lesson", "project", "plan",
    "schedule",
    // extended
    "mood", "vibe", "energy", "chill", "relax", "stress", "busy", "free", "available",
    "around", "nearby", "far", "miss", "care", "share", "send", "forward", "react", "reply",
    "quote", "mute", "block", "follow", "unfollow", "subscribe", "notify", "ping", "dm",
    "group", "channel", "thread", "status", "profile", "avatar", "emoji", "sticker", "gif",
    "photo", "selfie", "screenshot", "highlight", "story", "reel", "stream", "live", "post",
    "comment", "like", "share", "trending", "viral", "meme", "cringe", "based", "slay",
    "lowkey", "highkey", "literally", "honestly", "basically", "actually", "apparently",
    "totally", "absolutely", "definitely", "obviously", "clearly", "exactly", "precisely",
    "anyway", "whatever", "nevermind", "forget", "remember", "realize", "notice", "wonder",
    "guess", "suppose", "assume", "hope", "wish", "promise", "agree", "disagree", "debate",
    "argue", "joke", "tease", "flirt", "complain", "rant", "vent", "celebrate", "congrats",
    "birthday", "anniversary", "holiday", "vacation", "hangout", "brunch", "dinner",
];
