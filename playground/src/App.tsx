import { useEffect, useState } from "react";
import Editor from "@monaco-editor/react";
import initWasm, { LuminaRuntime } from "lumina-wasm";

import { StatePanel } from "./StatePanel";
import { AlertTimeline } from "./AlertTimeline";
import { VirtualClock } from "./VirtualClock";
import { ShareButton, loadFromURL } from "./ShareButton";
import "./App.css";

function App() {
    const [source, setSource] = useState(
        loadFromURL() || `entity Sensor { temp: Number }\nlet s1 = Sensor { temp: 20 }\nrule overheat {\n  when Sensor.temp > 40\n  then alert severity: "critical", message: "Too hot"\n}`
    );
    const [runtime, setRuntime] = useState<LuminaRuntime | null>(null);
    const [alerts, setAlerts] = useState<any[]>([]);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        initWasm().then(() => {
            compileAndRun(source);
        });
    }, []);

    const compileAndRun = (code: string) => {
        try {
            const err = LuminaRuntime.check(code);
            if (err) {
                setError(err);
                return;
            }
            const rt = new LuminaRuntime(code);
            setRuntime(rt);
            setError(null);
            setAlerts([]);
        } catch (e: any) {
            setError(e.toString());
        }
    };

    const handleRun = () => compileAndRun(source);

    const handleAlertsRaw = (tickResult: string) => {
        if (!tickResult) return;
        if (tickResult.startsWith("ERROR:")) {
            setError(tickResult.substring(6));
            return;
        }
        try {
            const evts = JSON.parse(tickResult);
            if (evts && evts.length > 0) {
                setAlerts(prev => [...prev, ...evts]);
            }
        } catch (e) {}
    };

    return (
        <div className="app-container">
            <header>
                <h1>Lumina Playground v2</h1>
                <div className="toolbar">
                    <button onClick={handleRun}>Compile & Run</button>
                    <VirtualClock rt={runtime} onAlerts={handleAlertsRaw} />
                    <ShareButton source={source} />
                </div>
            </header>
            <main>
                <div className="editor-pane">
                    <Editor
                        height="100%"
                        defaultLanguage="shell"
                        theme="vs-dark"
                        value={source}
                        onChange={(v) => setSource(v || "")}
                        options={{ minimap: { enabled: false } }}
                    />
                    {error && <div className="error-panel"><pre>{error}</pre></div>}
                </div>
                <div className="state-pane">
                    <StatePanel rt={runtime} />
                    <AlertTimeline events={alerts} />
                </div>
            </main>
        </div>
    );
}

export default App;
