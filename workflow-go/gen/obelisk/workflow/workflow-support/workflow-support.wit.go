// Code generated by wit-bindgen-go. DO NOT EDIT.

// Package workflowsupport represents the imported interface "obelisk:workflow/workflow-support@1.1.0".
package workflowsupport

import (
	"github.com/obeli-sk/demo-stargazers/workflow-go/gen/obelisk/types/execution"
	"github.com/obeli-sk/demo-stargazers/workflow-go/gen/obelisk/types/time"
	"go.bytecodealliance.org/cm"
)

// Duration represents the type alias "obelisk:workflow/workflow-support@1.1.0#duration".
//
// See [time.Duration] for more information.
type Duration = time.Duration

// JoinSetID represents the imported type alias "obelisk:workflow/workflow-support@1.1.0#join-set-id".
//
// See [execution.JoinSetID] for more information.
type JoinSetID = execution.JoinSetID

// ClosingStrategy represents the enum "obelisk:workflow/workflow-support@1.1.0#closing-strategy".
//
// The closing strategy of a join set. Join sets are closed when execution finishes.
//
//	enum closing-strategy {
//		complete
//	}
type ClosingStrategy uint8

const (
	// All submitted executions that were not awaited are awaited.
	ClosingStrategyComplete ClosingStrategy = iota
)

var _ClosingStrategyStrings = [1]string{
	"complete",
}

// String implements [fmt.Stringer], returning the enum case name of e.
func (e ClosingStrategy) String() string {
	return _ClosingStrategyStrings[e]
}

// MarshalText implements [encoding.TextMarshaler].
func (e ClosingStrategy) MarshalText() ([]byte, error) {
	return []byte(e.String()), nil
}

// UnmarshalText implements [encoding.TextUnmarshaler], unmarshaling into an enum
// case. Returns an error if the supplied text is not one of the enum cases.
func (e *ClosingStrategy) UnmarshalText(text []byte) error {
	return _ClosingStrategyUnmarshalCase(e, text)
}

var _ClosingStrategyUnmarshalCase = cm.CaseUnmarshaler[ClosingStrategy](_ClosingStrategyStrings[:])

// RandomU64 represents the imported function "random-u64".
//
// Returns a random u64 in the range [min, max).
//
//	random-u64: func(min: u64, max-exclusive: u64) -> u64
//
//go:nosplit
func RandomU64(min_ uint64, maxExclusive uint64) (result uint64) {
	min0 := (uint64)(min_)
	maxExclusive0 := (uint64)(maxExclusive)
	result0 := wasmimport_RandomU64((uint64)(min0), (uint64)(maxExclusive0))
	result = (uint64)((uint64)(result0))
	return
}

// RandomU64Inclusive represents the imported function "random-u64-inclusive".
//
// Returns a random u64 in the range [min, max].
//
//	random-u64-inclusive: func(min: u64, max-inclusive: u64) -> u64
//
//go:nosplit
func RandomU64Inclusive(min_ uint64, maxInclusive uint64) (result uint64) {
	min0 := (uint64)(min_)
	maxInclusive0 := (uint64)(maxInclusive)
	result0 := wasmimport_RandomU64Inclusive((uint64)(min0), (uint64)(maxInclusive0))
	result = (uint64)((uint64)(result0))
	return
}

// RandomString represents the imported function "random-string".
//
// Returns a random string with a length in the range [min_length, max_length).
// The string consists only of alphanumeric characters (lowercase and uppercase letters,
// digits).
//
//	random-string: func(min-length: u16, max-length-exclusive: u16) -> string
//
//go:nosplit
func RandomString(minLength uint16, maxLengthExclusive uint16) (result string) {
	minLength0 := (uint32)(minLength)
	maxLengthExclusive0 := (uint32)(maxLengthExclusive)
	wasmimport_RandomString((uint32)(minLength0), (uint32)(maxLengthExclusive0), &result)
	return
}

// Sleep represents the imported function "sleep".
//
// Persistent sleep.
//
//	sleep: func(duration: duration)
//
//go:nosplit
func Sleep(duration Duration) {
	duration0, duration1 := lower_Duration(duration)
	wasmimport_Sleep((uint32)(duration0), (uint64)(duration1))
	return
}

// NewJoinSetNamed represents the imported function "new-join-set-named".
//
// Create a new completing join set.
//
//	new-join-set-named: func(name: string, closing-strategy: closing-strategy) -> join-set-id
//
//go:nosplit
func NewJoinSetNamed(name string, closingStrategy ClosingStrategy) (result JoinSetID) {
	name0, name1 := cm.LowerString(name)
	closingStrategy0 := (uint32)(closingStrategy)
	result0 := wasmimport_NewJoinSetNamed((*uint8)(name0), (uint32)(name1), (uint32)(closingStrategy0))
	result = cm.Reinterpret[JoinSetID]((uint32)(result0))
	return
}

// NewJoinSetGenerated represents the imported function "new-join-set-generated".
//
// Create a new completing join set with a generated name.
//
//	new-join-set-generated: func(closing-strategy: closing-strategy) -> join-set-id
//
//go:nosplit
func NewJoinSetGenerated(closingStrategy ClosingStrategy) (result JoinSetID) {
	closingStrategy0 := (uint32)(closingStrategy)
	result0 := wasmimport_NewJoinSetGenerated((uint32)(closingStrategy0))
	result = cm.Reinterpret[JoinSetID]((uint32)(result0))
	return
}
