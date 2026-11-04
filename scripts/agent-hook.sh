#!/usr/bin/env bash
# Default agent hook for Claude Code and Codex: `guard` runs before an edit or
# shell command, `check` after an edit. The event arrives as JSON on stdin.
# The guard never runs Cargo, so a broken xtask/ cannot break it. Exit 2 blocks
# with the reason on stderr; exit 0 allows, with JSON on stdout when there is
# something to say. See docs/COMPLETION.md "Default hooks and CI".
set -uE
trap 'echo "agent-hook: internal error at line $LINENO. Tell the owner the hook needs a fix." >&2; exit 2' ERR

# Copied from xtask/src/transitions.rs; transitions::tests keeps them equal.
PROTECTED_PREFIXES="xtask/ .github/ scripts/ .cargo/ harness/goldens/ .claude/ .codex/"
PROTECTED_FILES="harness/config.json harness/readiness.json harness/tracking.json harness/workflows.json harness/scenarios.json Cargo.toml Cargo.lock docs/PRODUCT.md docs/READINESS.md docs/COMPLETION.md AGENTS.md"

mode=${1:-}
host=${2:-}
case "$mode/$host" in
  guard/claude | guard/codex | check/claude | check/codex) ;;
  *) echo "usage: agent-hook.sh guard|check claude|codex < event.json" >&2; exit 2 ;;
esac
event=$(head -c 1048576)

# Print the decoded string value of the first JSON key named $1, at any depth.
jget() {
  printf '%s' "$event" | awk -v want="$1" '
    function str(   out, c, h) {
      out = ""; i++
      while (i <= n) {
        c = substr(s, i, 1)
        if (c == "\"") { i++; return out }
        if (c == "\\") {
          i++; c = substr(s, i, 1)
          if (c == "n") out = out "\n"; else if (c == "t") out = out "\t"
          else if (c == "r") out = out "\r"; else if (c == "b" || c == "f") out = out " "
          else if (c == "u") {
            h = substr(s, i + 1, 4); i += 4
            if (h ~ /^00[0-7][0-9A-Fa-f]$/) out = out sprintf("%c", hex(h)); else out = out "?"
          } else out = out c
        } else out = out c
        i++
      }
      bad = 1; return out
    }
    function hex(h,   v, k, d) {
      v = 0
      for (k = 1; k <= 4; k++) { d = index("0123456789abcdef", tolower(substr(h, k, 1))) - 1; v = v * 16 + d }
      return v
    }
    function ws() { while (i <= n && substr(s, i, 1) ~ /[ \t\r\n]/) i++ }
    { s = s (NR > 1 ? "\n" : "") $0 }
    END {
      n = length(s); i = 1
      while (i <= n) {
        if (substr(s, i, 1) != "\"") { i++; continue }
        k = str(); ws()
        if (substr(s, i, 1) != ":") continue
        i++; ws()
        if (substr(s, i, 1) == "\"") { v = str(); if (k == want && !bad) { printf "%s", v; exit 0 } }
      }
    }'
}

json_string() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' | awk 'BEGIN{ORS=""} {print (NR > 1 ? "\\n" : "") $0}'; }

block() { echo "$1" >&2; exit 2; }
allow() { printf '{}\n'; exit 0; }
note() {
  printf '{"hookSpecificOutput":{"hookEventName":"%s","additionalContext":"%s"}}\n' "$1" "$(json_string "$2")"
  exit 0
}
ask() {
  if [ "$host" = codex ]; then block "$2"; fi
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"ask","permissionDecisionReason":"%s"}}\n' "$(json_string "$1")"
  exit 0
}

tool=$(jget tool_name) || tool=""
cwd=$(jget cwd) || cwd=""
[ -n "$cwd" ] || cwd=$PWD
[ -n "$tool" ] || block "Blocked: agent-hook could not read this event. Tell the owner the hook needs a fix."
# Outside a Git checkout nothing here applies.
root=$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null) || allow
prefix=$(git -C "$cwd" rev-parse --show-prefix 2>/dev/null) || prefix=""

# The same root can be spelled /private/var/... (Git) or /var/... (the event).
root_cwd=${cwd%/}
[ -z "$prefix" ] || root_cwd=${root_cwd%/"${prefix%/}"}
root_short=${root#/private}

# Repository-relative path for $1, or "" when it lies outside the checkout.
relative() {
  local p=$1 r
  case "$p" in
    /*)
      for r in "$root" "$root_cwd" "$root_short" "/private$root_cwd"; do
        case "$p" in "$r"/*) p=${p#"$r"/}; break ;; esac
      done
      case "$p" in /*) echo ""; return ;; esac ;;
    *) p="$prefix$p" ;;
  esac
  while :; do case "$p" in ./*) p=${p#./} ;; *) break ;; esac; done
  case "/$p/" in */../*) echo "UNSAFE:$1"; return ;; esac
  echo "$p"
}

# Tier of one repository-relative path: golden, deps, policy or none.
tier() {
  local p=$1 x
  case "$p" in "" ) echo none; return ;; harness/goldens | harness/goldens/*) echo golden; return ;; harness/config.json) echo deps; return ;; esac
  for x in $PROTECTED_PREFIXES; do case "$p" in "$x"*) echo policy; return ;; esac; done
  for x in $PROTECTED_FILES; do [ "$p" = "$x" ] && { echo policy; return; }; done
  echo none
}

rank() { case "$1" in golden) echo 3 ;; deps) echo 2 ;; policy) echo 1 ;; *) echo 0 ;; esac; }

# Relative paths for raw paths given one per line in $1.
relatives() {
  local out="" line
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    out="$out$(relative "$line")
"
  done <<EOF
$1
EOF
  printf '%s' "$out"
}

# Decide for repository-relative paths given one per line in $1.
decide() {
  local best=none best_path="" p t
  while IFS= read -r p; do
    [ -n "$p" ] || continue
    case "$p" in UNSAFE:*) block "Blocked: agent-hook will not judge a path containing '..' (${p#UNSAFE:}). Use a path inside the repository." ;; esac
    t=$(tier "$p")
    if [ "$(rank "$t")" -gt "$(rank "$best")" ]; then best=$t; best_path=$p; fi
  done <<EOF
$1
EOF
  case "$best" in
    golden)
      ask "$best_path is a golden baseline. Approve only a reviewed baseline change; the branch will also need a policy transition receipt." \
        "Blocked: $best_path is a golden baseline and Codex hooks cannot ask. Fix the implementation, or ask the owner to change the baseline by hand or from Claude Code." ;;
    deps)
      ask "$best_path holds the per-layer dependency allowlists that ARCH004 checks. Widening one adds a dependency, which the owner approves." \
        "Blocked: $best_path holds the dependency allowlists and Codex hooks cannot ask. Ask the owner to widen it." ;;
    policy)
      note PreToolUse "$best_path is a protected policy file. This branch will need a transition receipt: run cargo xtask gaps --base origin/main before completion." ;;
    *) allow ;;
  esac
}

# Shell commands: print "merge" for PR merges and force-pushes, and "write WORD"
# for each word a command writes to. Quoted words and heredoc bodies stay whole,
# so a commit message or a fixture that only mentions a path is not a write.
shell_targets() {
  printf '%s' "$1" | awk '
    function flush_word() { if (inword) { w[++nw] = word; tgt[nw] = redir; redir = 0 } word = ""; inword = 0 }
    function flush_seg(   k, j, cmd, sub1, ops, nops, hase, inplace, arg, force) {
      flush_word()
      for (k = 1; k <= nw; k++) if (tgt[k]) print "write " w[k]
      k = 1
      while (k <= nw && (tgt[k] || w[k] ~ /^[A-Za-z_][A-Za-z0-9_]*=/ || w[k] ~ /^(sudo|env|command|nohup|time|exec|xargs)$/)) k++
      cmd = w[k]; sub(/.*\//, "", cmd); nops = 0; hase = 0; force = 0
      if (cmd == "git") {
        k++
        while (k <= nw && w[k] ~ /^-/) { if (w[k] == "-C" || w[k] == "-c") k++; k++ }
        sub1 = w[k]
        for (j = k + 1; j <= nw; j++) {
          if (tgt[j]) continue
          arg = w[j]
          if (sub1 == "push" && (arg ~ /^--force/ || arg ~ /^-[A-Za-z]*f[A-Za-z]*$/ || arg ~ /^\+/)) force = 1
          if ((sub1 == "checkout" || sub1 == "restore") && arg !~ /^-/) print "write " arg
        }
        if (force) print "merge"
      } else if (cmd == "gh") {
        if (w[k + 1] == "pr" && w[k + 2] == "merge") print "merge"
        if (w[k + 1] == "api") for (j = k + 2; j <= nw; j++) if (w[j] ~ /\/merge([?\/]|$)/) print "merge"
      } else if (cmd ~ /^(mv|rm|tee|truncate|cp|install|sed)$/) {
        inplace = 0
        for (j = k + 1; j <= nw; j++) {
          if (tgt[j]) continue
          arg = w[j]
          if (cmd == "sed" && (arg ~ /^-i/ || arg ~ /^--in-place/)) { inplace = 1; continue }
          if (cmd == "sed" && (arg == "-e" || arg == "-f")) { hase = 1; j++; continue }
          if ((cmd == "cp" || cmd == "install") && arg == "-t") { j++; print "write " w[j]; continue }
          if ((cmd == "cp" || cmd == "install") && arg ~ /^--target-directory=/) { sub(/^--target-directory=/, "", arg); print "write " arg; continue }
          if (arg ~ /^-/ && arg != "-") continue
          ops[++nops] = arg
        }
        if (cmd == "cp" || cmd == "install") { if (nops >= 2) print "write " ops[nops] }
        else if (cmd == "sed") { if (inplace) for (j = (hase ? 1 : 2); j <= nops; j++) print "write " ops[j] }
        else for (j = 1; j <= nops; j++) print "write " ops[j]
      }
      for (k = 1; k <= nw; k++) { delete w[k]; delete tgt[k] }
      nw = 0
    }
    { lines[NR] = $0 }
    END {
      # Drop heredoc bodies: everything after <<WORD up to a line equal to WORD.
      text = ""; delim = ""
      for (r = 1; r <= NR; r++) {
        line = lines[r]
        if (delim != "") { t = line; sub(/^\t+/, "", t); if (t == delim) delim = ""; continue }
        text = text line "\n"
        l2 = line; gsub(/<<</, "   ", l2)
        if (match(l2, /<<-?[ \t]*["\047]?[A-Za-z_][A-Za-z0-9_]*["\047]?/)) {
          delim = substr(l2, RSTART, RLENGTH); sub(/^<<-?[ \t]*/, "", delim); gsub(/["\047]/, "", delim)
        }
      }
      n = length(text); q = ""; word = ""; inword = 0; redir = 0; nw = 0
      for (i = 1; i <= n; i++) {
        c = substr(text, i, 1)
        if (q == "\047") { if (c == "\047") q = ""; else word = word c; continue }
        if (q == "\"") {
          if (c == "\\" && i < n) { i++; word = word substr(text, i, 1); continue }
          if (c == "\"") q = ""; else word = word c
          continue
        }
        if (c == "\\" && i < n) { i++; word = word substr(text, i, 1); inword = 1; continue }
        if (c == "\047" || c == "\"") { q = c; inword = 1; continue }
        if (c == ">") {
          if (inword && word ~ /^[0-9&]*$/) { word = ""; inword = 0 } else flush_word()
          while (substr(text, i + 1, 1) ~ /[>|]/) i++
          if (substr(text, i + 1, 1) == "&") { i++; continue }
          redir = 1; continue
        }
        if (c == "<") { flush_word(); while (substr(text, i + 1, 1) ~ /[<&]/) i++; skipnext = 1; continue }
        if (c == ";" || c == "|" || c == "&" || c == "\n" || c == "(" || c == ")") { flush_seg(); continue }
        if (c == " " || c == "\t") { if (inword && skipnext) { word = ""; inword = 0; skipnext = 0 } else flush_word(); continue }
        word = word c; inword = 1
      }
      flush_seg()
    }'
}

guard() {
  local paths="" cmd found
  case "$tool" in
    Edit | Write | MultiEdit)
      paths=$(jget file_path) || paths=""
      [ -n "$paths" ] || block "Blocked: agent-hook could not read the path of this edit. Tell the owner the hook needs a fix."
      decide "$(relatives "$paths")" ;;
    NotebookEdit)
      paths=$(jget notebook_path) || paths=""
      [ -n "$paths" ] || block "Blocked: agent-hook could not read the path of this edit. Tell the owner the hook needs a fix."
      decide "$(relatives "$paths")" ;;
    apply_patch)
      cmd=$(jget command) || cmd=""
      paths=$(printf '%s\n' "$cmd" | sed -n -e 's/^\*\*\* Add File: //p' -e 's/^\*\*\* Update File: //p' -e 's/^\*\*\* Delete File: //p' -e 's/^\*\*\* Move to: //p')
      [ -n "$paths" ] || block "Blocked: agent-hook could not read the paths in this patch. Edit one file per patch, or tell the owner the hook needs a fix."
      decide "$(relatives "$paths")" ;;
    Bash)
      cmd=$(jget command) || cmd=""
      found=$(shell_targets "$cmd")
      if printf '%s\n' "$found" | grep -qx merge; then
        block "Blocked: force-pushing and merging the PR need the owner (AGENTS.md, Operating rules). Push without force, or ask the owner."
      fi
      decide "$(relatives "$(printf '%s\n' "$found" | sed -n 's/^write //p')")" ;;
    *) allow ;;
  esac
}

case "$mode" in
  guard) guard ;;
  check) allow ;;
esac
