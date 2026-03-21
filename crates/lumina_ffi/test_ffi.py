"""
Run with: python test_ffi.py
Requires: cargo build --release -p lumina_ffi
"""
from lumina_py import LuminaRuntime

SOURCE = """
entity Moto {
  battery: Number
  isBusy: Boolean
  isLowBattery := battery < 20
  isAvailable  := not isBusy and battery > 15
}

rule "low battery alert" {
  when Moto.isLowBattery becomes true
  then show "Python FFI Test: battery low!"
}

let Moto = Moto { battery: 80, isBusy: false }
"""


def test_create():
    rt = LuminaRuntime.from_source(SOURCE)
    state = rt.export_state()
    assert "Moto" in str(state)
    print("✓ test_create")


def test_apply_event():
    rt = LuminaRuntime.from_source(SOURCE)
    result = rt.apply_event("Moto", "battery", 10)
    state = rt.export_state()
    instances = state["instances"]
    moto = instances["Moto"]
    assert moto["fields"]["battery"] == 10
    assert moto["fields"]["isLowBattery"] == True
    assert moto["fields"]["isAvailable"] == False
    print("✓ test_apply_event")


def test_derived_recomputes():
    rt = LuminaRuntime.from_source(SOURCE)
    rt.apply_event("Moto", "battery", 50)
    state = rt.export_state()
    moto = state["instances"]["Moto"]
    assert moto["fields"]["isAvailable"] == True
    assert moto["fields"]["isLowBattery"] == False
    print("✓ test_derived_recomputes")


def test_rollback_message():
    rt = LuminaRuntime.from_source(SOURCE)
    try:
        rt.apply_event("Moto", "isLowBattery", True)
        print("✗ test_rollback_message — expected error, got none")
    except RuntimeError as e:
        assert "Cannot update derived field" in str(e)
        assert "computed automatically" in str(e)
        print(f"✓ test_rollback_message — got expected error message with help")


def test_get_messages():
    rt = LuminaRuntime.from_source(SOURCE)
    rt.apply_event("Moto", "battery", 10)
    messages = rt.get_messages()
    assert len(messages) == 1
    assert "battery low!" in messages[0]
    print("✓ test_get_messages")


def test_creation_error():
    try:
        LuminaRuntime.from_source("entity { broken")
        print("✗ test_creation_error — expected error, got none")
    except ValueError as e:
        assert "Parse error" in str(e)
        print(f"✓ test_creation_error — got descriptive error: {e}")


if __name__ == "__main__":
    print("Running Lumina FFI tests...\n")
    test_create()
    test_apply_event()
    test_derived_recomputes()
    test_rollback_message()
    test_get_messages()
    test_creation_error()
    print("\nAll FFI tests passed ✓")
