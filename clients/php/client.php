<?php class Client { public function healthPath(string $base): string { return rtrim($base, '/').'/v1/health'; } }
