package wasihttp

import (
	"errors"
	"fmt"
	"io"
	"net/http"
	"runtime"
	"sync"

	"go.bytecodealliance.org/cm"
	"github.com/obeli-sk/demo-stargazers/openai-go/gen/wasi/http/types"
	"github.com/obeli-sk/demo-stargazers/openai-go/gen/wasi/io/streams"
)

// BodyConsumer interface is implemented by [types.IncomingRequest] and [types.IncomingResponse].
// It enables the consumption of [wasi:http/types.incoming-request] and [wasi:http/types.incoming-request]
//
// [wasi:http/types.incoming-request]: https://github.com/WebAssembly/wasi-http/blob/v0.2.0/wit/types.wit#L220-L248
//
// [wasi:http/types.incoming-request]: https://github.com/WebAssembly/wasi-http/blob/v0.2.0/wit/types.wit#L377-L395
type BodyConsumer interface {
	Consume() (result cm.Result[types.IncomingBody, types.IncomingBody, struct{}])
	Headers() (result types.Fields)
}

type inputStreamReader struct {
	consumer    BodyConsumer
	body        *types.IncomingBody
	stream      *streams.InputStream
	trailerLock sync.Mutex
	trailers    http.Header
	trailerOnce sync.Once
}

func (r *inputStreamReader) Close() error {
	r.trailerOnce.Do(r.parseTrailers)

	if r.stream != nil {
		r.stream.ResourceDrop()
	}

	if r.body != nil {
		r.body.ResourceDrop()
		r.body = nil
	}

	return nil
}

func (r *inputStreamReader) parseTrailers() {
	r.trailerLock.Lock()
	defer r.trailerLock.Unlock()

	// if we got this far, then we release ownership from body, otherwise it is our responsibility to drop it
	r.stream.ResourceDrop()
	r.stream = nil

	futureTrailers := types.IncomingBodyFinish(*r.body)
	defer futureTrailers.ResourceDrop()

	trailersResult := futureTrailers.Get()
	r.body = nil

	// unroll the future
	if trailersResult.None() {
		return
	}
	if trailersResult.Some().IsErr() {
		return
	}
	if trailersResult.Some().OK().IsErr() {
		return
	}
	maybeWasiTrailers := trailersResult.Some().OK().OK()

	if maybeWasiTrailers.None() {
		return
	}

	wasiTrailers := maybeWasiTrailers.Some()
	for _, kv := range wasiTrailers.Entries().Slice() {
		r.trailers.Add(string(kv.F0), string(kv.F1.Slice()))
	}

	wasiTrailers.ResourceDrop()
}

func (r *inputStreamReader) Read(p []byte) (n int, err error) {
	pollable := r.stream.Subscribe()
	for !pollable.Ready() {
		runtime.Gosched()
	}
	pollable.ResourceDrop()

	readResult := r.stream.Read(uint64(len(p)))
	if err := readResult.Err(); err != nil {
		if err.Closed() {
			r.trailerOnce.Do(r.parseTrailers)
			return 0, io.EOF
		}
		return 0, fmt.Errorf("failed to read from InputStream %s", err.LastOperationFailed().ToDebugString())
	}

	readList := *readResult.OK()
	copy(p, readList.Slice())
	return int(readList.Len()), nil
}

// NewIncomingBodyTrailer takes a [BodyConsumer] and parses it into corresponding [io.ReadCloser] and [net/http.Header].
func NewIncomingBodyTrailer(consumer BodyConsumer) (io.ReadCloser, http.Header, error) {
	consumeResult := consumer.Consume()
	if consumeResult.IsErr() {
		return nil, nil, errors.New("failed to consume incoming request")
	}

	body := consumeResult.OK()
	streamResult := body.Stream()
	if streamResult.IsErr() {
		return nil, nil, errors.New("failed to consume incoming request body stream")
	}

	stream := streamResult.OK()

	trailers := http.Header{}
	return &inputStreamReader{
		consumer: consumer,
		trailers: trailers,
		body:     body,
		stream:   stream,
	}, trailers, nil
}

type outgoingBody struct {
	body   *types.OutgoingBody
	stream *streams.OutputStream
}

// NewOutgoingBody takes a [types.OutgoingBody] and returns a [io.WriteCloser] encapsulating it.
func NewOutgoingBody(body *types.OutgoingBody) (io.WriteCloser, error) {
	stream := body.Write()
	if stream.IsErr() {
		return nil, errors.New("failed to acquire resource handle to request body")
	}
	return &outgoingBody{
		body:   body,
		stream: stream.OK(),
	}, nil
}

func (r *outgoingBody) Close() error {
	r.stream.ResourceDrop()
	return nil
}

func (r *outgoingBody) Write(p []byte) (n int, err error) {
	contents := cm.ToList(p)
	writeResult := r.stream.BlockingWriteAndFlush(contents)
	if err := writeResult.Err(); err != nil {
		if err.Closed() {
			return 0, io.EOF
		}
		return 0, fmt.Errorf("failed to write to response body's stream: %s", err.LastOperationFailed().ToDebugString())
	}
	return len(p), nil
}
