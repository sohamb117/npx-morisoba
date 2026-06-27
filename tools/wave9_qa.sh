#!/usr/bin/env bash
# Wave 9 scenario verification harness for the SSH Profile TUI.
#
# Reproduces the seven manual scenarios (S1..S7) that gate "done" on this project:
#   S1 first-paint, S2 navigation, S3 clean-quit,
#   S4 TMUX -> Ascii, S5 missing hero, S6 resize, S7 zero-color SGR.
#
# Each scenario runs inside its own tmux socket (-L S1, -L S2, ...) so they cannot
# collide. Captured artifacts are written under $QA for human inspection.
#
# Usage:
#   ./tools/wave9_qa.sh
#
# Environment overrides:
#   WORKDIR  - repo root (default: git toplevel, else $PWD)
#   BIN      - binary path (default: $WORKDIR/target/release/ssh-profile-tui)
#   QA       - artifact dir (default: $WORKDIR/.qa)
#
# Requirements: bash 4+, tmux, the binary built with `cargo build --release`.

set -u
WORKDIR="${WORKDIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
BIN="${BIN:-$WORKDIR/target/release/morisoba}"
QA="${QA:-$WORKDIR/.qa}"

mkdir -p "$QA"
cd "$WORKDIR" || { echo "FATAL: cannot cd to $WORKDIR"; exit 9; }
[ -x "$BIN" ] || {
    echo "FATAL: binary missing or not executable at $BIN"
    echo "Build first with: cargo build --release"
    exit 9
}
command -v tmux >/dev/null 2>&1 || {
    echo "FATAL: tmux not found in PATH"
    exit 9
}

PASS=0
FAIL=0
declare -A RESULT
mark() {
    local name=$1 outcome=$2
    RESULT[$name]="$outcome"
    [ "$outcome" = PASS ] && PASS=$((PASS+1)) || FAIL=$((FAIL+1))
    printf "  %s\n" "$name $outcome"
}

# ---------------- S1 first-paint ----------------
echo "================ S1 first-paint ================"
tmux -L S1 kill-server 2>/dev/null
tmux -L S1 new-session -d -s tui -x 120 -y 40
tmux -L S1 send-keys -t tui "$BIN" Enter
sleep 1.0
tmux -L S1 capture-pane -t tui -p    > "$QA/S1.txt"
tmux -L S1 capture-pane -t tui -p -e > "$QA/S1.ansi"
S1OK=1
grep -q "MORISOBA"            "$QA/S1.txt" || { echo "  miss MORISOBA";   S1OK=0; }
for w in "ABOUT" "PROJECTS" "EXPERIENCE" "CONTACT"; do
    grep -q "$w" "$QA/S1.txt" || { echo "  miss nav $w"; S1OK=0; }
done
grep -q "Q QUIT"              "$QA/S1.txt" || { echo "  miss Q QUIT";      S1OK=0; }
grep -q "// HERO"             "$QA/S1.txt" || { echo "  miss // HERO";     S1OK=0; }
grep -q "// NAV"              "$QA/S1.txt" || { echo "  miss // NAV";      S1OK=0; }
grep -q "// CONTENT"          "$QA/S1.txt" || { echo "  miss // CONTENT";  S1OK=0; }
grep -q "brutalist aesthetic" "$QA/S1.txt" || { echo "  miss About body";  S1OK=0; }
[ $S1OK -eq 1 ] && mark S1 PASS || mark S1 FAIL
tmux -L S1 kill-server 2>/dev/null

# ---------------- S2 navigation ----------------
echo "================ S2 navigation ================"
tmux -L S2 kill-server 2>/dev/null
tmux -L S2 new-session -d -s tui -x 120 -y 40
tmux -L S2 send-keys -t tui "$BIN" Enter
sleep 1.0
tmux -L S2 capture-pane -t tui -p > "$QA/S2_init.txt"
tmux -L S2 send-keys -t tui Down ; sleep 0.5
tmux -L S2 capture-pane -t tui -p > "$QA/S2_d.txt"
tmux -L S2 send-keys -t tui Down ; sleep 0.5
tmux -L S2 capture-pane -t tui -p > "$QA/S2_dd.txt"
tmux -L S2 send-keys -t tui Up   ; sleep 0.5
tmux -L S2 capture-pane -t tui -p > "$QA/S2_ddu.txt"
S2OK=1
grep -q "brutalist aesthetic"    "$QA/S2_init.txt" || { echo "  init miss About";       S2OK=0; }
grep -q "DISTRIBUTED LOG ENGINE" "$QA/S2_d.txt"    || { echo "  +Down miss Projects";   S2OK=0; }
grep -q "brutalist aesthetic"    "$QA/S2_d.txt"    && { echo "  +Down About lingers";   S2OK=0; }
grep -q "SENIOR SWE"             "$QA/S2_dd.txt"   || { echo "  +DD miss Experience";   S2OK=0; }
grep -q "DISTRIBUTED LOG ENGINE" "$QA/S2_ddu.txt"  || { echo "  +DDU miss Projects";    S2OK=0; }
[ $S2OK -eq 1 ] && mark S2 PASS || mark S2 FAIL
tmux -L S2 kill-server 2>/dev/null

# ---------------- S3 clean quit ----------------
echo "================ S3 clean quit ================"
tmux -L S3 kill-server 2>/dev/null
tmux -L S3 new-session -d -s tui -x 120 -y 40
tmux -L S3 send-keys -t tui "$BIN" Enter
sleep 1.0
tmux -L S3 capture-pane -t tui -p > "$QA/S3_before.txt"
tmux -L S3 send-keys -t tui q ; sleep 1.0
tmux -L S3 capture-pane -t tui -p > "$QA/S3_after.txt"
tmux -L S3 send-keys -t tui "echo SHELL_RESPONSIVE_OK" Enter ; sleep 0.4
tmux -L S3 capture-pane -t tui -p > "$QA/S3_shell.txt"
S3OK=1
grep -q "// NAV" "$QA/S3_before.txt" || { echo "  sanity NAV missing BEFORE q";          S3OK=0; }
grep -q "// NAV" "$QA/S3_after.txt"  && { echo "  NAV still visible AFTER q";            S3OK=0; }
grep -q "SHELL_RESPONSIVE_OK" "$QA/S3_shell.txt" || { echo "  shell unresponsive";       S3OK=0; }
[ $S3OK -eq 1 ] && mark S3 PASS || mark S3 FAIL
tmux -L S3 kill-server 2>/dev/null

# ---------------- S4 TMUX -> ASCII ----------------
echo "================ S4 TMUX -> ASCII ================"
tmux -L S4 kill-server 2>/dev/null
tmux -L S4 new-session -d -s tui -x 120 -y 40
tmux -L S4 send-keys -t tui "TERM=xterm-kitty $BIN" Enter
sleep 1.0
tmux -L S4 capture-pane -t tui -p    > "$QA/S4.txt"
tmux -L S4 capture-pane -t tui -p -e > "$QA/S4.ansi"
S4OK=1
grep -q "MORISOBA"             "$QA/S4.txt"  || { echo "  binary did not render";              S4OK=0; }
grep -Pq '\x1b_G'              "$QA/S4.ansi" && { echo "  kitty graphics escape leaked";        S4OK=0; }
grep -Pq '\x1b\]1337;File='    "$QA/S4.ansi" && { echo "  iTerm inline image escape leaked";    S4OK=0; }
[ $S4OK -eq 1 ] && mark S4 PASS || mark S4 FAIL
tmux -L S4 kill-server 2>/dev/null

# ---------------- S5 missing hero ----------------
echo "================ S5 missing hero ================"
HERO_BACKUP=""
if [ -f "$WORKDIR/assets/hero.png" ]; then
    HERO_BACKUP="/tmp/hero_backup_$$.png"
    mv "$WORKDIR/assets/hero.png" "$HERO_BACKUP"
fi
tmux -L S5 kill-server 2>/dev/null
tmux -L S5 new-session -d -s tui -x 120 -y 40
tmux -L S5 send-keys -t tui "$BIN" Enter
sleep 1.0
tmux -L S5 capture-pane -t tui -p > "$QA/S5.txt"
S5OK=1
grep -q "\[ HERO \]" "$QA/S5.txt" || { echo "  no placeholder text"; S5OK=0; }
grep -q "// HERO"    "$QA/S5.txt" || { echo "  hero border missing"; S5OK=0; }
ALIVE=$(tmux -L S5 list-panes -t tui -F '#{pane_dead}' 2>/dev/null)
[ "$ALIVE" = "0" ] || { echo "  pane reports dead (pane_dead=$ALIVE)"; S5OK=0; }
[ $S5OK -eq 1 ] && mark S5 PASS || mark S5 FAIL
tmux -L S5 kill-server 2>/dev/null
if [ -n "$HERO_BACKUP" ] && [ -f "$HERO_BACKUP" ]; then
    mv "$HERO_BACKUP" "$WORKDIR/assets/hero.png"
fi

# ---------------- S6 resize ----------------
echo "================ S6 resize ================"
tmux -L S6 kill-server 2>/dev/null
tmux -L S6 new-session -d -s tui -x 80 -y 24
tmux -L S6 send-keys -t tui "$BIN" Enter
sleep 1.0
tmux -L S6 capture-pane -t tui -p > "$QA/S6_80x24.txt"
tmux -L S6 resize-window -t tui -x 160 -y 50
sleep 0.7
tmux -L S6 capture-pane -t tui -p > "$QA/S6_160x50.txt"
S6OK=1
for f in "$QA/S6_80x24.txt" "$QA/S6_160x50.txt"; do
    bn=$(basename "$f")
    grep -q "MORISOBA" "$f" || { echo "  miss MORISOBA in $bn"; S6OK=0; }
    grep -q "ABOUT"    "$f" || { echo "  miss ABOUT in $bn";    S6OK=0; }
    grep -q "Q QUIT"   "$f" || { echo "  miss Q QUIT in $bn";   S6OK=0; }
done
[ $S6OK -eq 1 ] && mark S6 PASS || mark S6 FAIL
tmux -L S6 kill-server 2>/dev/null

# ---------------- S7 zero color SGR ----------------
echo "================ S7 zero color SGR ================"
tmux -L S7 kill-server 2>/dev/null
tmux -L S7 new-session -d -s tui -x 120 -y 40
tmux -L S7 send-keys -t tui "$BIN" Enter
sleep 1.0
tmux -L S7 capture-pane -t tui -p -e > "$QA/S7.ansi"
S7OK=1
if grep -Pq '\x1b\[(?:[0-9;]*;)?(3[0-7]|4[0-7]|9[0-7]|10[0-7]|38;5|48;5|38;2|48;2)' "$QA/S7.ansi"; then
    echo "  forbidden color SGR present. Distinct codes:"
    grep -Po '\x1b\[[0-9;]*m' "$QA/S7.ansi" | sort -u | head -20
    S7OK=0
else
    echo "  Allowed SGRs only. Distinct codes:"
    grep -Po '\x1b\[[0-9;]*m' "$QA/S7.ansi" | sort -u | head -20
fi
[ $S7OK -eq 1 ] && mark S7 PASS || mark S7 FAIL
tmux -L S7 kill-server 2>/dev/null

# ---------------- S8 'i' key smoke (does not crash, pane stays alive) ----------------
echo "================ S8 'i' key smoke ================"
tmux -L S8 kill-server 2>/dev/null
tmux -L S8 new-session -d -s tui -x 120 -y 40
tmux -L S8 send-keys -t tui "$BIN" Enter
sleep 1.0
tmux -L S8 capture-pane -t tui -p > "$QA/S8_before.txt"
tmux -L S8 send-keys -t tui i
sleep 0.6
tmux -L S8 capture-pane -t tui -p > "$QA/S8_after.txt"
S8OK=1
ALIVE=$(tmux -L S8 list-panes -t tui -F '#{pane_dead}' 2>/dev/null)
[ "$ALIVE" = "0" ] || { echo "  pane reports dead (pane_dead=$ALIVE) after pressing i"; S8OK=0; }
grep -q "// NAV"     "$QA/S8_after.txt" || { echo "  // NAV missing after pressing i";     S8OK=0; }
grep -q "I IMAGE"    "$QA/S8_after.txt" || { echo "  I IMAGE footer hint missing";         S8OK=0; }
tmux -L S8 send-keys -t tui q
sleep 0.5
[ $S8OK -eq 1 ] && mark S8 PASS || mark S8 FAIL
tmux -L S8 kill-server 2>/dev/null

# ---------------- summary ----------------
echo ""
echo "================ WAVE 9 SUMMARY ================"
for k in S1 S2 S3 S4 S5 S6 S7 S8; do printf "  %s = %s\n" "$k" "${RESULT[$k]:-SKIPPED}"; done
echo ""
echo "PASS=$PASS  FAIL=$FAIL"
echo "Artifacts written to: $QA"
exit $FAIL
