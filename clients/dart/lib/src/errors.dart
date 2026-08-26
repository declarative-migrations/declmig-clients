class ClientException implements Exception {
  const ClientException(this.code);
  final String code;
  @override
  String toString() => 'ClientException($code)';
}

