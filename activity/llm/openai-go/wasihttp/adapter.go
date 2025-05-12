package wasihttp

// Refactored from https://github.com/rajatjindal/wasi-go-sdk/tree/d3e8665bef9fbf0794ad14f7114a9882e0d983c3/pkg/wasihttp
// and https://github.com/wasmCloud/go/tree/4b0e28296bcc7aa6caba7439afd0f0180396866f
import (
	"fmt"
	"io"
	"net/http"
	"sync"

	"go.bytecodealliance.org/cm"
	"github.com/obeli-sk/demo-stargazers/openai-go/gen/wasi/http/types"
	"github.com/obeli-sk/demo-stargazers/openai-go/gen/wasi/io/streams"
)

var _ http.ResponseWriter = (*ResponseOutparamWriter)(nil)

// IncomingRequest represents an incoming HTTP request as defined in [wasi:http/types.incoming-request]
//
// [wasi:http/types.incoming-request]: https://github.com/WebAssembly/wasi-http/blob/v0.2.0/wit/types.wit#L220-L248
type IncomingRequest = types.IncomingRequest

// ResponseOutparamWriter implements a [net/http.ResponseWriter] for [wasi:http]
//
// [wasi:http]: https://github.com/WebAssembly/wasi-http/tree/v0.2.0
type ResponseOutparamWriter struct {
	outparam    types.ResponseOutparam
	response    types.OutgoingResponse
	wasiHeaders types.Fields
	httpHeaders http.Header
	body        *types.OutgoingBody
	stream      *streams.OutputStream

	headerOnce sync.Once
	headerErr  error

	statuscode int
}

// Header returns the header map that will be sent by [ResponseOutparamWriter.WriteHeader].
func (row *ResponseOutparamWriter) Header() http.Header {
	return row.httpHeaders
}

// Write writes the data to the connection as part of an HTTP reply.
func (row *ResponseOutparamWriter) Write(buf []byte) (int, error) {
	// NOTE(lxf): If this is the first write, make sure we set the headers/statuscode
	row.headerOnce.Do(row.reconcile)
	if row.headerErr != nil {
		return 0, row.headerErr
	}

	contents := cm.ToList(buf)
	writeResult := row.stream.Write(contents)
	if writeResult.IsErr() {
		if writeResult.Err().Closed() {
			return 0, io.EOF
		}

		return 0, fmt.Errorf("failed to write to response body's stream: %s", writeResult.Err().LastOperationFailed().ToDebugString())
	}

	row.stream.BlockingFlush()

	return int(contents.Len()), nil
}

// WriteHeader sends an HTTP response header with the provided
// status code.
func (row *ResponseOutparamWriter) WriteHeader(statusCode int) {
	row.headerOnce.Do(func() {
		row.statuscode = statusCode
		row.reconcile()
	})
}

// reconcile headers from go to wasi
func (row *ResponseOutparamWriter) reconcileHeaders() error {
	for key, vals := range row.httpHeaders {
		fieldVals := []types.FieldValue{}
		for _, val := range vals {
			fieldVals = append(fieldVals, types.FieldValue(cm.ToList([]uint8(val))))
		}

		if result := row.wasiHeaders.Set(types.FieldKey(key), cm.ToList(fieldVals)); result.IsErr() {
			return fmt.Errorf("failed to set header %s: %s", key, result.Err())
		}
	}

	// NOTE(lxf): once headers are written we clear them out so they can emit http trailers
	row.httpHeaders = http.Header{}

	return nil
}

func (row *ResponseOutparamWriter) reconcile() {
	if row.headerErr = row.reconcileHeaders(); row.headerErr != nil {
		return
	}

	row.response = types.NewOutgoingResponse(row.wasiHeaders)
	row.response.SetStatusCode(types.StatusCode(row.statuscode))

	bodyResult := row.response.Body()
	if bodyResult.IsErr() {
		row.headerErr = fmt.Errorf("failed to acquire resource handle to response body: %s", bodyResult.Err())
		return
	}
	row.body = bodyResult.OK()

	writeResult := row.body.Write()
	if writeResult.IsErr() {
		row.headerErr = fmt.Errorf("failed to acquire resource handle for response body's stream: %s", writeResult.Err())
		return
	}
	row.stream = writeResult.OK()

	result := cm.OK[cm.Result[types.ErrorCodeShape, types.OutgoingResponse, types.ErrorCode]](row.response)
	types.ResponseOutparamSet(row.outparam, result)
}

// Close closes out the underlying stream by flushing the response and making
// sure that the underlying resource handle is dropped.
func (row *ResponseOutparamWriter) Close() error {
	if row.stream == nil {
		return nil
	}

	row.stream.BlockingFlush()
	row.stream.ResourceDrop()
	row.stream = nil

	var maybeTrailers cm.Option[types.Fields]
	wasiTrailers := types.NewFields()
	for key, vals := range row.httpHeaders {
		fieldVals := []types.FieldValue{}
		for _, val := range vals {
			fieldVals = append(fieldVals, types.FieldValue(cm.ToList([]uint8(val))))
		}

		if result := wasiTrailers.Set(types.FieldKey(key), cm.ToList(fieldVals)); result.IsErr() {
			return fmt.Errorf("failed to set trailer %s: %s", key, result.Err())
		}
	}
	if len(row.httpHeaders) > 0 {
		maybeTrailers = cm.Some(wasiTrailers)
	} else {
		maybeTrailers = cm.None[types.Fields]()
	}

	res := types.OutgoingBodyFinish(*row.body, maybeTrailers)
	if res.IsErr() {
		return fmt.Errorf("failed to set trailer: %v", res.Err())
	}
	return nil
}

// WASItoHTTPResponseWriter takes a [types.ResponseOutparam] representing [wasi:http/types.response-outparam]
// and instantiates a new [ResponseOutparamWriter] for writing to it.
//
// [wasi:http/types.response-outparam]: https://github.com/WebAssembly/wasi-http/blob/v0.2.0/wit/types.wit#L352-L372
func WASItoHTTPResponseWriter(out types.ResponseOutparam) *ResponseOutparamWriter {
	return &ResponseOutparamWriter{
		outparam:    out,
		httpHeaders: http.Header{},
		wasiHeaders: types.NewFields(),
		statuscode:  http.StatusOK,
	}
}

// WASItoHTTPRequest takes an [IncomingRequest] and returns a [net/http.Request] representation of it.
func WASItoHTTPRequest(ir IncomingRequest) (req *http.Request, err error) {
	method, err := methodToString(ir.Method())
	if err != nil {
		return nil, err
	}

	authority := "localhost"
	if auth := ir.Authority(); !auth.None() {
		authority = *auth.Some()
	}

	pathWithQuery := "/"
	if p := ir.PathWithQuery(); !p.None() {
		pathWithQuery = *p.Some()
	}

	body, trailers, err := NewIncomingBodyTrailer(ir)
	if err != nil {
		switch method {
		case http.MethodGet,
			http.MethodHead,
			http.MethodDelete,
			http.MethodConnect,
			http.MethodOptions,
			http.MethodTrace:
		default:
			return nil, fmt.Errorf("failed to consume incoming request: %w", err)
		}
	}

	url := fmt.Sprintf("http://%s%s", authority, pathWithQuery)
	req, err = http.NewRequest(method, url, body)
	if err != nil {
		return nil, err
	}
	req.Trailer = trailers

	headers := ir.Headers()
	WASItoHTTPHeader(headers, &req.Header)
	headers.ResourceDrop()

	req.Host = authority
	req.URL.Host = authority
	req.RequestURI = pathWithQuery

	return req, nil
}

func methodToString(m types.Method) (string, error) {
	if m.Connect() {
		return http.MethodConnect, nil
	} else if m.Delete() {
		return http.MethodDelete, nil
	} else if m.Get() {
		return http.MethodGet, nil
	} else if m.Head() {
		return http.MethodHead, nil
	} else if m.Options() {
		return http.MethodOptions, nil
	} else if m.Patch() {
		return http.MethodPatch, nil
	} else if m.Post() {
		return http.MethodPost, nil
	} else if m.Put() {
		return http.MethodPut, nil
	} else if m.Trace() {
		return http.MethodTrace, nil
	} else if other := m.Other(); other != nil {
		return *other, fmt.Errorf("unknown http method '%s'", *other)
	}
	return "", fmt.Errorf("failed to convert http method")
}

// WASItoHTTPHeader takes a [types.Fields] and copies them to the provided [net/http.Header] map.
func WASItoHTTPHeader(src types.Fields, dest *http.Header) {
	for _, f := range src.Entries().Slice() {
		key := string(f.F0)
		value := string(cm.List[uint8](f.F1).Slice())
		dest.Add(key, value)
	}
}

// HTTPtoWASIHeader takes a [net/http.Header] map and copies them to the provided [types.Fields].
func HTTPtoWASIHeader(src http.Header, dest types.Fields) error {
	for k, v := range src {
		key := types.FieldKey(k)
		fieldVals := []types.FieldValue{}

		for _, val := range v {
			fieldVals = append(fieldVals, types.FieldValue(cm.ToList([]uint8(val))))
		}

		res := dest.Set(key, cm.ToList(fieldVals))
		if res.IsErr() {
			return fmt.Errorf("failed to set header %s: %s", k, res.Err())
		}
	}

	return nil
}

func toWasiMethod(s string) types.Method {
	switch s {
	case http.MethodConnect:
		return types.MethodConnect()
	case http.MethodDelete:
		return types.MethodDelete()
	case http.MethodGet:
		return types.MethodGet()
	case http.MethodHead:
		return types.MethodHead()
	case http.MethodOptions:
		return types.MethodOptions()
	case http.MethodPatch:
		return types.MethodPatch()
	case http.MethodPost:
		return types.MethodPost()
	case http.MethodPut:
		return types.MethodPut()
	case http.MethodTrace:
		return types.MethodTrace()
	default:
		return types.MethodOther(s)
	}
}
