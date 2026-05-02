#!/usr/bin/env nu
# session-summary.nu — cross-session triage: what the last session left behind.
# Usage: nu session-summary.nu [--sessions 3]

use ($"(git rev-parse --show-toplevel | str trim)/skills/_lib/helpers.nu") *

def main [--sessions: int = 3] {
    let events = (open-trace)

    # Collect unique session_ids in order of first appearance
    let session_ids = ($events
        | get session_id
        | uniq
        | last $sessions)

    for sid in $session_ids {
        let evts = ($events | where session_id == $sid)
        let started = ($evts | first | get ts)
        let errors  = ($evts | where event == "skill.error"   | length)
        let blocked = ($evts | where event == "agent.blocked" | length)
        let complete = ($evts | where event == "agent.complete" | length)
        let running = ($evts
            | where event == "agent.start"
            | where { |e|
                not ($evts | any { |x| $x.event == "agent.complete" and $x.agent_id == $e.agent_id })
                and not ($evts | any { |x| $x.event == "agent.blocked" and $x.agent_id == $e.agent_id })
            }
            | length)
        let decisions = ($evts | where event == "decision" | length)

        print $"--- ($sid) @ ($started)"
        print $"    errors=($errors)  blocked=($blocked)  agents: ($complete) complete / ($running) still-running / ($decisions) decisions"

        # Surface any unresolved running tasks at session end
        if $running > 0 {
            let running_agents = ($evts
                | where event == "agent.start"
                | where { |e|
                    not ($evts | any { |x| $x.event == "agent.complete" and $x.agent_id == $e.agent_id })
                    and not ($evts | any { |x| $x.event == "agent.blocked" and $x.agent_id == $e.agent_id })
                })
            for a in $running_agents {
                print $"    UNRESOLVED agent: ($a.agent_id) slot=($a.slot)"
            }
        }
    }
}
