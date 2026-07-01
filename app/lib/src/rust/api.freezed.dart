// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'api.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
  'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models',
);

/// @nodoc
mixin _$LevelValue {
  num get field0 => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(double field0) float,
    required TResult Function(int field0) int,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(double field0)? float,
    TResult? Function(int field0)? int,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(double field0)? float,
    TResult Function(int field0)? int,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LevelValue_Float value) float,
    required TResult Function(LevelValue_Int value) int,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LevelValue_Float value)? float,
    TResult? Function(LevelValue_Int value)? int,
  }) => throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LevelValue_Float value)? float,
    TResult Function(LevelValue_Int value)? int,
    required TResult orElse(),
  }) => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $LevelValueCopyWith<$Res> {
  factory $LevelValueCopyWith(
    LevelValue value,
    $Res Function(LevelValue) then,
  ) = _$LevelValueCopyWithImpl<$Res, LevelValue>;
}

/// @nodoc
class _$LevelValueCopyWithImpl<$Res, $Val extends LevelValue>
    implements $LevelValueCopyWith<$Res> {
  _$LevelValueCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of LevelValue
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$LevelValue_FloatImplCopyWith<$Res> {
  factory _$$LevelValue_FloatImplCopyWith(
    _$LevelValue_FloatImpl value,
    $Res Function(_$LevelValue_FloatImpl) then,
  ) = __$$LevelValue_FloatImplCopyWithImpl<$Res>;
  @useResult
  $Res call({double field0});
}

/// @nodoc
class __$$LevelValue_FloatImplCopyWithImpl<$Res>
    extends _$LevelValueCopyWithImpl<$Res, _$LevelValue_FloatImpl>
    implements _$$LevelValue_FloatImplCopyWith<$Res> {
  __$$LevelValue_FloatImplCopyWithImpl(
    _$LevelValue_FloatImpl _value,
    $Res Function(_$LevelValue_FloatImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of LevelValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null}) {
    return _then(
      _$LevelValue_FloatImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                  as double,
      ),
    );
  }
}

/// @nodoc

class _$LevelValue_FloatImpl extends LevelValue_Float {
  const _$LevelValue_FloatImpl(this.field0) : super._();

  @override
  final double field0;

  @override
  String toString() {
    return 'LevelValue.float(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$LevelValue_FloatImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of LevelValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$LevelValue_FloatImplCopyWith<_$LevelValue_FloatImpl> get copyWith =>
      __$$LevelValue_FloatImplCopyWithImpl<_$LevelValue_FloatImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(double field0) float,
    required TResult Function(int field0) int,
  }) {
    return float(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(double field0)? float,
    TResult? Function(int field0)? int,
  }) {
    return float?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(double field0)? float,
    TResult Function(int field0)? int,
    required TResult orElse(),
  }) {
    if (float != null) {
      return float(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LevelValue_Float value) float,
    required TResult Function(LevelValue_Int value) int,
  }) {
    return float(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LevelValue_Float value)? float,
    TResult? Function(LevelValue_Int value)? int,
  }) {
    return float?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LevelValue_Float value)? float,
    TResult Function(LevelValue_Int value)? int,
    required TResult orElse(),
  }) {
    if (float != null) {
      return float(this);
    }
    return orElse();
  }
}

abstract class LevelValue_Float extends LevelValue {
  const factory LevelValue_Float(final double field0) = _$LevelValue_FloatImpl;
  const LevelValue_Float._() : super._();

  @override
  double get field0;

  /// Create a copy of LevelValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$LevelValue_FloatImplCopyWith<_$LevelValue_FloatImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$LevelValue_IntImplCopyWith<$Res> {
  factory _$$LevelValue_IntImplCopyWith(
    _$LevelValue_IntImpl value,
    $Res Function(_$LevelValue_IntImpl) then,
  ) = __$$LevelValue_IntImplCopyWithImpl<$Res>;
  @useResult
  $Res call({int field0});
}

/// @nodoc
class __$$LevelValue_IntImplCopyWithImpl<$Res>
    extends _$LevelValueCopyWithImpl<$Res, _$LevelValue_IntImpl>
    implements _$$LevelValue_IntImplCopyWith<$Res> {
  __$$LevelValue_IntImplCopyWithImpl(
    _$LevelValue_IntImpl _value,
    $Res Function(_$LevelValue_IntImpl) _then,
  ) : super(_value, _then);

  /// Create a copy of LevelValue
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({Object? field0 = null}) {
    return _then(
      _$LevelValue_IntImpl(
        null == field0
            ? _value.field0
            : field0 // ignore: cast_nullable_to_non_nullable
                  as int,
      ),
    );
  }
}

/// @nodoc

class _$LevelValue_IntImpl extends LevelValue_Int {
  const _$LevelValue_IntImpl(this.field0) : super._();

  @override
  final int field0;

  @override
  String toString() {
    return 'LevelValue.int(field0: $field0)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$LevelValue_IntImpl &&
            (identical(other.field0, field0) || other.field0 == field0));
  }

  @override
  int get hashCode => Object.hash(runtimeType, field0);

  /// Create a copy of LevelValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$LevelValue_IntImplCopyWith<_$LevelValue_IntImpl> get copyWith =>
      __$$LevelValue_IntImplCopyWithImpl<_$LevelValue_IntImpl>(
        this,
        _$identity,
      );

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(double field0) float,
    required TResult Function(int field0) int,
  }) {
    return int(field0);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(double field0)? float,
    TResult? Function(int field0)? int,
  }) {
    return int?.call(field0);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(double field0)? float,
    TResult Function(int field0)? int,
    required TResult orElse(),
  }) {
    if (int != null) {
      return int(field0);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(LevelValue_Float value) float,
    required TResult Function(LevelValue_Int value) int,
  }) {
    return int(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(LevelValue_Float value)? float,
    TResult? Function(LevelValue_Int value)? int,
  }) {
    return int?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(LevelValue_Float value)? float,
    TResult Function(LevelValue_Int value)? int,
    required TResult orElse(),
  }) {
    if (int != null) {
      return int(this);
    }
    return orElse();
  }
}

abstract class LevelValue_Int extends LevelValue {
  const factory LevelValue_Int(final int field0) = _$LevelValue_IntImpl;
  const LevelValue_Int._() : super._();

  @override
  int get field0;

  /// Create a copy of LevelValue
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$LevelValue_IntImplCopyWith<_$LevelValue_IntImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
