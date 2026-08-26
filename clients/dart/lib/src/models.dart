class Health {
  const Health({required this.ok, required this.service});
  final bool ok;
  final String service;
}

class ResourceEnvelope {
  const ResourceEnvelope({required this.id, required this.revision, required this.payload});
  final String id;
  final String revision;
  final Map<String, Object?> payload;
  static const resource = 'MigrationPlan';
}

