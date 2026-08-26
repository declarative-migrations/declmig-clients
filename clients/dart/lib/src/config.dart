import 'errors.dart';

class ClientConfig {
  const ClientConfig({required this.baseUrl, this.bearerToken, this.maxResponseBytes = 64 * 1024});
  final String baseUrl;
  final String? bearerToken;
  final int maxResponseBytes;

  factory ClientConfig.fromEnvironment(Map<String, String> env) {
    final base = env['DECLMIG_API_BASE']?.trim() ?? '';
    if (base.isEmpty) {
      throw const ClientException('invalid_base');
    }
    return ClientConfig(baseUrl: base, bearerToken: env['DECLMIG_TOKEN']);
  }
}

