import LZString from "lz-string";

export function ShareButton({ source }: { source: string }) {
    const share = () => {
        const enc = LZString.compressToEncodedURIComponent(source);
        const url = `${location.origin}/play#v=2&src=${enc}`;
        navigator.clipboard.writeText(url);
        alert("Link copied!");
    };
    return <button onClick={share}>Share</button>;
}

export function loadFromURL(): string | null {
    const m = location.hash.match(/[#&]src=([^&]*)/);
    return m ? LZString.decompressFromEncodedURIComponent(m[1]) : null;
}
