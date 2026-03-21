package lumina_test

import (
	"testing"
	lumina "." // import the local package
)

const basicSource = `
entity Moto {
	battery: Number
	isLowBattery := battery < 20
}
let moto1 = Moto { battery: 80 }
`

func TestFromSource(t *testing.T) {
	rt, err := lumina.FromSource(basicSource)
	if err != nil {
		t.Fatalf("FromSource failed: %v", err)
	}
	defer rt.Close()
}

func TestFromSourceInvalid(t *testing.T) {
	_, err := lumina.FromSource("this is not valid lumina @@@@")
	if err == nil {
		t.Fatal("expected error for invalid source")
	}
}

func TestApplyEventAndExportState(t *testing.T) {
	rt, err := lumina.FromSource(basicSource)
	if err != nil {
		t.Fatal(err)
	}
	defer rt.Close()
	_, err = rt.ApplyEvent("moto1", "battery", "15")
	if err != nil {
		t.Fatalf("ApplyEvent failed: %v", err)
	}
	state, err := rt.ExportState()
	if err != nil {
		t.Fatalf("ExportState failed: %v", err)
	}
	instances := state["instances"].(map[string]any)
	moto1 := instances["moto1"].(map[string]any)
	fields := moto1["fields"].(map[string]any)
	isLow := fields["isLowBattery"].(bool)
	if !isLow {
		t.Error("expected isLowBattery=true after setting battery=15")
	}
}

func TestRollbackOnDerivedField(t *testing.T) {
	rt, err := lumina.FromSource(basicSource)
	if err != nil {
		t.Fatal(err)
	}
	defer rt.Close()
	_, err = rt.ApplyEvent("moto1", "isLowBattery", "true")
	if err == nil {
		t.Fatal("expected rollback error R009 for derived field write")
	}
}

func TestTick(t *testing.T) {
	rt, err := lumina.FromSource(basicSource)
	if err != nil {
		t.Fatal(err)
	}
	defer rt.Close()
	events, err := rt.Tick()
	if err != nil {
		t.Fatalf("Tick failed: %v", err)
	}
	// No every/for rules in basicSource - events should be empty
	if len(events) != 0 {
		t.Errorf("expected 0 events, got %d", len(events))
	}
}
