import { useEffect, useRef, useState } from "react";
import type { LuminaRuntime } from "lumina-wasm";

export function VirtualClock({
    rt, onAlerts
}: { rt: LuminaRuntime | null; onAlerts: (a: string) => void }) {
    const [speed, setSpeed] = useState(1);
    const [running, setRunning] = useState(false);
    const ref = useRef<number | null>(null);

    useEffect(() => {
        if (!running || !rt) return;
        ref.current = setInterval(() => {
            const evts = rt.tick();
            if (evts?.length) onAlerts(evts);
        }, 100 / speed) as unknown as number;
        return () => clearInterval(ref.current!);
    }, [running, speed, rt]);

    return (
        <div className="clock">
            <button onClick={() => setRunning(r => !r)}>
                {running ? "Pause" : "Run"}
            </button>
            {[1, 10, 100].map(s => (
                <button key={s}
                    className={speed === s ? "active" : ""}
                    onClick={() => setSpeed(s)}>{s}x</button>
            ))}
        </div>
    );
}
