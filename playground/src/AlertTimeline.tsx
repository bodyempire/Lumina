interface Entry { severity: string; source?: string; message: string; rule: string; ts: number; }

const COLORS: Record<string,string> = {
    critical:"#FEE2E2", warning:"#FEF9C3", info:"#DBEAFE", resolved:"#DCFCE7"
};

export function AlertTimeline({ events }: { events: Entry[] }) {
    return (
        <div className="timeline">
            <h4>Alert Timeline</h4>
            {events.length === 0 && <p>No alerts fired</p>}
            {[...events].reverse().map((e, i) => (
                <div key={i} style={{ background: COLORS[e.severity] || "#F5F5F5" }}>
                    <span>{e.severity.toUpperCase()}</span>
                    <span>{e.source || e.rule}</span>
                    <span>{e.message}</span>
                </div>
            ))}
        </div>
    );
}
