// Code generated by wit-bindgen-go. DO NOT EDIT.

// Package log represents the imported interface "obelisk:log/log@1.0.0".
package log

import (
	"go.bytecodealliance.org/cm"
)

// Trace represents the imported function "trace".
//
//	trace: func(message: string)
//
//go:nosplit
func Trace(message string) {
	message0, message1 := cm.LowerString(message)
	wasmimport_Trace((*uint8)(message0), (uint32)(message1))
	return
}

// Debug represents the imported function "debug".
//
//	debug: func(message: string)
//
//go:nosplit
func Debug(message string) {
	message0, message1 := cm.LowerString(message)
	wasmimport_Debug((*uint8)(message0), (uint32)(message1))
	return
}

// Info represents the imported function "info".
//
//	info: func(message: string)
//
//go:nosplit
func Info(message string) {
	message0, message1 := cm.LowerString(message)
	wasmimport_Info((*uint8)(message0), (uint32)(message1))
	return
}

// Warn represents the imported function "warn".
//
//	warn: func(message: string)
//
//go:nosplit
func Warn(message string) {
	message0, message1 := cm.LowerString(message)
	wasmimport_Warn((*uint8)(message0), (uint32)(message1))
	return
}

// Error represents the imported function "error".
//
//	error: func(message: string)
//
//go:nosplit
func Error(message string) {
	message0, message1 := cm.LowerString(message)
	wasmimport_Error((*uint8)(message0), (uint32)(message1))
	return
}
