import 'dart:convert';
import 'dart:typed_data';

import 'config.dart';
import 'errors.dart';
import 'models.dart';

class Client {
  Client(this.config) {
    if (config.baseUrl.trim().isEmpty) {
      throw const ClientException('invalid_base');
    }
  }

  final ClientConfig config;

  String healthUrl() => '${config.baseUrl.replaceAll(RegExp(r'/$'), '')}/v1/health';

  Health decodeHealth(Uint8List body) {
    if (body.length > config.maxResponseBytes) {
      throw const ClientException('too_large');
    }
    final decoded = jsonDecode(utf8.decode(body));
    if (decoded is! Map<String, Object?>) {
      throw const ClientException('invalid_json');
    }
    return Health(ok: decoded['ok'] == true, service: '${decoded['service']}');
  }
}

