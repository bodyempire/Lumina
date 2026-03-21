import { useEffect, useState } from "react";
import type { LuminaRuntime } from "lumina-wasm";

interface Field { name: string; value: unknown; isDerived: boolean; }
interface Card { name: string; fields: Field[]; hasAlert: boolean; }

export function StatePanel({ rt }: { rt: LuminaRuntime | null }) {
    const [cards, setCards] = useState<Card[]>([]);

    useEffect(() => {
        if (!rt) return;
        const id = setInterval(() => {
            const stateJson = rt.export_state();
            if (!stateJson) return;
            const state = JSON.parse(stateJson);
            if (!state?.instances) return;
            setCards(Object.entries(state.instances).map(([name, inst]: any) => ({
                name,
                hasAlert: inst.active_alert ?? false,
                fields: Object.entries(inst.fields || {}).map(([f, v]) => ({
                    name: f, value: v,
                    isDerived: inst.derived_fields?.includes(f) ?? false
                }))
            })));
        }, 200);
        return () => clearInterval(id);
    }, [rt]);

    return (
        <div className="state-panel">
            {cards.map(c => (
                <div key={c.name} className={"card" + (c.hasAlert ? " alert" : "")}>
                    <h3>{c.name}{c.hasAlert && <span className="badge">ALERT</span>}</h3>
                    {c.fields.map(f => (
                        <div key={f.name} className={f.isDerived ? "derived" : "stored"}>
                            <span>{f.name}</span><span>{String(f.value)}</span>
                        </div>
                    ))}
                </div>
            ))}
        </div>
    );
}
