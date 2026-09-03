package validationconsumer

import (
	"testing"
	public "github.com/declarative-migrations/declmig-lib-core/validation/golang"
)

func TestValidatesBeforeTransport(t *testing.T) {
	_, err := ValidatedRequestHeaders(public.RequestMeta{RequestID: "req-1", TraceID: "trace-1"})
	if err != nil { t.Fatalf("unexpected validation failure: %v", err) }
}
