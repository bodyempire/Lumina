package lumina

/*
#cgo linux LDFLAGS: -L../../../target/release -L../../../target/debug -llumina_ffi -Wl,-rpath,../../../target/release -Wl,-rpath,../../../target/debug
#cgo darwin LDFLAGS: -L../../../target/release -L../../../target/debug -llumina_ffi
#include "../lumina.h"
#include <stdlib.h>
*/
import "C"

import (
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"unsafe"
)

// Runtime wraps the opaque LuminaRuntime C pointer.
// Always call Close() when done - it frees the C-side memory.
type Runtime struct {
	ptr *C.LuminaRuntime
}

// FromSource creates a Runtime from a Lumina source string.
// Returns an error if parsing or analysis fails.
func FromSource(source string) (*Runtime, error) {
	cs := C.CString(source)
	defer C.free(unsafe.Pointer(cs))
	ptr := C.lumina_create(cs)
	if ptr == nil {
		return nil, errors.New("lumina: failed to create runtime (parse/analyze error)")
	}
	return &Runtime{ptr: ptr}, nil
}

// ApplyEvent sets instance.field = value.
// valueJSON must be a valid JSON-encoded string:
// Number: "42" or "3.14"
// Text: "\"hello\"" (extra quotes - it is JSON)
// Boolean: "true" or "false"
// Returns the PropResult as a map, or an error on rollback.
func (r *Runtime) ApplyEvent(instance, field, valueJSON string) (map[string]any, error) {
	ci := C.CString(instance)
	cf := C.CString(field)
	cv := C.CString(valueJSON)
	defer C.free(unsafe.Pointer(ci))
	defer C.free(unsafe.Pointer(cf))
	defer C.free(unsafe.Pointer(cv))
	raw := C.lumina_apply_event(r.ptr, ci, cf, cv)
	if raw == nil {
		return nil, errors.New("lumina: null response from apply_event")
	}
	defer C.lumina_free_string(raw)
	result := C.GoString(raw)
	// Rollback is signaled by "ERROR:{...}"
	if strings.HasPrefix(result, "ERROR:") {
		return nil, fmt.Errorf("lumina rollback: %s", result[6:])
	}
	var out map[string]any
	if err := json.Unmarshal([]byte(result), &out); err != nil {
		return nil, fmt.Errorf("lumina: cannot parse response: %w", err)
	}
	return out, nil
}

// ExportState returns the full runtime state as a parsed map.
// Keys: "instances" -> map of instance name -> { "fields": {...} }
func (r *Runtime) ExportState() (map[string]any, error) {
	raw := C.lumina_export_state(r.ptr)
	if raw == nil {
		return nil, errors.New("lumina: null response from export_state")
	}
	defer C.lumina_free_string(raw)
	var out map[string]any
	err := json.Unmarshal([]byte(C.GoString(raw)), &out)
	return out, err
}

// Tick advances all timers. Returns a slice of fired events.
// Call on a time.Ticker for every/for rules.
func (r *Runtime) Tick() ([]map[string]any, error) {
	raw := C.lumina_tick(r.ptr)
	if raw == nil {
		return nil, errors.New("lumina: null response from tick")
	}
	defer C.lumina_free_string(raw)
	result := C.GoString(raw)
	if strings.HasPrefix(result, "ERROR:") {
		return nil, fmt.Errorf("lumina rollback: %s", result[6:])
	}
	var events []map[string]any
	if err := json.Unmarshal([]byte(result), &events); err != nil {
		return nil, fmt.Errorf("lumina: cannot parse response: %w", err)
	}
	return events, nil
}

// GetMessages retrieves any strings printed by rule actions.
func (r *Runtime) GetMessages() ([]string, error) {
	raw := C.lumina_get_messages(r.ptr)
	if raw == nil {
		return nil, errors.New("lumina: null response from get_messages")
	}
	defer C.lumina_free_string(raw)
	var messages []string
	if err := json.Unmarshal([]byte(C.GoString(raw)), &messages); err != nil {
		return nil, fmt.Errorf("lumina: cannot parse messages: %w", err)
	}
	return messages, nil
}

// Close destroys the runtime and frees all C-side memory.
// After Close(), the Runtime must not be used.
func (r *Runtime) Close() {
	if r.ptr != nil {
		C.lumina_destroy(r.ptr)
		r.ptr = nil
	}
}
