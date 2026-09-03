package validationconsumer

import public "github.com/declarative-migrations/declmig-lib-core/validation/golang"

func ValidatedRequestHeaders(meta public.RequestMeta) (map[string]string, error) {
	if err := public.Validate(meta); err != nil { return nil, err }
	headers := map[string]string{"x-request-id": meta.RequestID, "traceparent": meta.TraceID}
	if meta.Locale != "" { headers["accept-language"] = meta.Locale }
	return headers, nil
}

func ValidateProblem(problem public.ProblemDetails) error { return public.Validate(problem) }
