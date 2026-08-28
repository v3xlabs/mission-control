#!/usr/bin/env bash
# Writes an .ics into the lab's media directory with events a few minutes out, so the rail, the
# countdown and both toast leads can be watched without waiting for a real meeting.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
media="$root/.tmp/lab/config/media"
offset="${1:-6}"

mkdir -p "$media"

stamp() { date -u -d "+$1 minutes" +%Y%m%dT%H%M%SZ; }

cat > "$media/lab.ics" <<ICS
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//missiond//lab//EN
BEGIN:VEVENT
UID:lab-standup
SUMMARY:Standup
LOCATION:Room 2
DTSTART:$(stamp "$offset")
DTEND:$(stamp $((offset + 15)))
END:VEVENT
BEGIN:VEVENT
UID:lab-review
SUMMARY:Design review
LOCATION:Room 4
DTSTART:$(stamp $((offset + 40)))
DTEND:$(stamp $((offset + 100)))
END:VEVENT
BEGIN:VEVENT
UID:lab-weekly
SUMMARY:Weekly sync
LOCATION:Online
DTSTART:$(stamp $((offset + 180)))
DTEND:$(stamp $((offset + 210)))
RRULE:FREQ=DAILY;COUNT=3
END:VEVENT
END:VCALENDAR
ICS

echo "wrote $media/lab.ics, first event in $offset minutes"
