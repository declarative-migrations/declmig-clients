class Client:
    def health_url(self, base: str) -> str:
        return f"{base.rstrip('/')}/v1/health"
