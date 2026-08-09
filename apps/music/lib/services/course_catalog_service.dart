// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../courses/course_manifest.dart';
import '../src/grpc/score.pbgrpc.dart' as score;
import 'grpc_client.dart';
import 'preferences_service.dart';

part 'course_catalog_service.g.dart';

/// Seam over the backend's `ScoreService.ListCourses` / `GetCourse` (change:
/// add-notation-courses): the server-stored interactive courses.
///
/// The app sources all course content from here — never bundled — so a course is
/// data on the server. Behind a provider so tests inject a fake set (or an
/// error) without a backend. Both methods **throw** on a transport failure; the
/// [coursesProvider] / [courseManifestProvider] layer reconciles that with a
/// local cache for offline use.
abstract class CourseCatalogService {
  /// The published courses' listing metadata (grouped/ordered server-side).
  Future<List<CourseListing>> listCourses();

  /// The raw manifest JSON for [id] (the opaque `content_json` blob), or null
  /// when the id is unknown/unpublished. Parsing is the caller's job so the raw
  /// blob can be cached verbatim for offline replay.
  Future<String?> getCourseManifestJson(String id);
}

/// Production [CourseCatalogService] over the generated `ScoreServiceClient`,
/// authenticated like the other music RPCs.
class GrpcCourseCatalogService implements CourseCatalogService {
  GrpcCourseCatalogService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = score.ScoreServiceClient(channel),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<List<CourseListing>> listCourses() => _authed((bearer) async {
    final resp = await _client.listCourses(
      score.ListCoursesRequest(),
      options: bearerOptions(bearer),
    );
    return resp.courses.map(_listingOf).toList();
  });

  @override
  Future<String?> getCourseManifestJson(String id) => _authed((bearer) async {
    final resp = await _client.getCourse(
      score.GetCourseRequest(id: id),
      options: bearerOptions(bearer),
    );
    return resp.hasCourse() ? resp.course.contentJson : null;
  });

  static CourseListing _listingOf(score.CourseSummary s) => CourseListing(
    id: s.id,
    schemaVersion: s.schemaVersion,
    instrument: s.instrument,
    track: s.track,
    level: s.level,
    unit: s.unit,
    unitTitle: parseInlineJson(s.unitTitleJson),
    sortOrder: s.sortOrder,
    title: parseInlineJson(s.titleJson),
  );
}

/// Production course-catalog service provider. Override in tests with a fake.
@riverpod
CourseCatalogService courseCatalogService(Ref ref) => GrpcCourseCatalogService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);

/// Seam over the backend's cross-device course completion (change:
/// add-notation-courses): `ScoreService.RecordCourseCompletion` / `GetCourseProgress`.
/// Separate from [CourseCatalogService] so the completion notifier (and tests)
/// depend only on what they use. Owner-scoped; both methods **throw** off-line /
/// signed-out, which the notifier treats as "local only".
abstract class CourseProgressService {
  /// The ids of the courses the signed-in user has completed (any device).
  Future<Set<String>> completedCourseIds();

  /// Records a completion of [courseId] for the signed-in user (idempotent
  /// server-side; the first one awards the badge).
  Future<void> recordCompletion(String courseId);
}

/// Production [CourseProgressService] over the generated `ScoreServiceClient`.
class GrpcCourseProgressService implements CourseProgressService {
  GrpcCourseProgressService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = score.ScoreServiceClient(channel),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<Set<String>> completedCourseIds() => _authed((bearer) async {
    final resp = await _client.getCourseProgress(
      score.GetCourseProgressRequest(),
      options: bearerOptions(bearer),
    );
    return resp.progress
        .where((p) => p.completed)
        .map((p) => p.courseId)
        .toSet();
  });

  @override
  Future<void> recordCompletion(String courseId) => _authed((bearer) async {
    await _client.recordCourseCompletion(
      score.RecordCourseCompletionRequest(courseId: courseId),
      options: bearerOptions(bearer),
    );
  });
}

/// Production course-progress service provider. Override in tests with a fake.
@riverpod
CourseProgressService courseProgressService(Ref ref) =>
    GrpcCourseProgressService(
      channel: ref.watch(cymbraChannelProvider),
      authed: ref.watch(authedRunnerProvider),
    );

const String _kCoursesCacheKey = 'courses.listing.v1';
String _manifestCacheKey(String id) => 'courses.manifest.v1.$id';

/// Drops the listings this build cannot run (a `schemaVersion` above what the
/// parser understands) — otherwise a newer server course would render a
/// tappable tile leading to a dead player screen.
List<CourseListing> _supported(List<CourseListing> listings) =>
    listings.where((l) => l.schemaVersion <= kCourseSchemaVersion).toList();

/// The course listing, reconciled with an offline cache: fetches from the
/// server and refreshes the cache, or — when the server is unreachable — falls
/// back to the last cached listing (empty if none). Never throws. Listings the
/// build cannot run are filtered out (the cache keeps them, so an app update
/// reveals them without a refetch).
@riverpod
Future<List<CourseListing>> courses(Ref ref) async {
  final prefs = ref.watch(preferencesServiceProvider);
  try {
    final list = await ref.watch(courseCatalogServiceProvider).listCourses();
    try {
      await prefs.setString(_kCoursesCacheKey, _encodeListings(list));
    } catch (_) {}
    return _supported(list);
  } catch (_) {
    final cached = await _readCache(prefs, _kCoursesCacheKey);
    return cached == null ? const [] : _supported(_decodeListings(cached));
  }
}

/// A single course's parsed manifest, reconciled with an offline cache: fetches
/// the raw blob and refreshes the cache, or falls back to the last cached blob.
/// Null when the course is unknown or its `schemaVersion` is unsupported.
@riverpod
Future<CourseManifest?> courseManifest(Ref ref, String id) async {
  final prefs = ref.watch(preferencesServiceProvider);
  try {
    final raw = await ref
        .watch(courseCatalogServiceProvider)
        .getCourseManifestJson(id);
    if (raw == null) return null;
    try {
      await prefs.setString(_manifestCacheKey(id), raw);
    } catch (_) {}
    return parseCourseManifest(raw);
  } catch (_) {
    final cached = await _readCache(prefs, _manifestCacheKey(id));
    return cached == null ? null : parseCourseManifest(cached);
  }
}

Future<String?> _readCache(PreferencesService prefs, String key) async {
  try {
    return await prefs.getString(key);
  } catch (_) {
    return null;
  }
}

/// Encodes listings for the cache, keeping the title as its raw inline-i18n map.
String _encodeListings(List<CourseListing> listings) => jsonEncode([
  for (final l in listings)
    {
      'id': l.id,
      'schemaVersion': l.schemaVersion,
      'instrument': l.instrument,
      'track': l.track,
      'level': l.level,
      'unit': l.unit,
      'unitTitle': l.unitTitle,
      'sortOrder': l.sortOrder,
      'title': l.title,
    },
]);

InlineText _cachedInline(Object? raw) => {
  if (raw is Map)
    for (final entry in raw.entries)
      if (entry.key is String && entry.value is String)
        entry.key as String: entry.value as String,
};

List<CourseListing> _decodeListings(String source) {
  try {
    final decoded = jsonDecode(source);
    if (decoded is! List) return const [];
    return [
      for (final e in decoded)
        if (e is Map && e['id'] is String && e['schemaVersion'] is int)
          CourseListing(
            id: e['id'] as String,
            schemaVersion: e['schemaVersion'] as int,
            instrument: e['instrument'] is String
                ? e['instrument'] as String
                : 'piano',
            track: e['track'] is String ? e['track'] as String : 'solfege',
            level: e['level'] is String ? e['level'] as String : 'beginner',
            unit: e['unit'] is String ? e['unit'] as String : '',
            unitTitle: _cachedInline(e['unitTitle']),
            sortOrder: e['sortOrder'] is int ? e['sortOrder'] as int : 0,
            title: _cachedInline(e['title']),
          ),
    ];
  } catch (_) {
    return const [];
  }
}
