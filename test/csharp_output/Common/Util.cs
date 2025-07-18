using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading.Tasks;

namespace Cfg.Common;

public static class Util {
    public static JsonSerializerOptions Options = new JsonSerializerOptions() {
        Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
        IncludeFields = true,
        WriteIndented = true,
        // 似乎 .net 9 才支持这个选项, godot 还没法用. 
        // 不过我已经调整过 $type 的位置, 到最开头了
        // 所以没关系, 之前加是为了预防意外
        // AllowOutOfOrderMetadataProperties = true,
        Converters = {
            new JsonStringEnumConverter(),
            new DictionaryTKeyObjectTValueConverter()
        }
    };
}

public class DictionaryTKeyObjectTValueConverter : JsonConverterFactory {
    public override bool CanConvert(Type typeToConvert) {
        if (!typeToConvert.IsGenericType) {
            return false;
        }

        if (typeToConvert.GetGenericTypeDefinition() != typeof(Dictionary<,>)) {
            return false;
        }

        return true;
    }

    public override JsonConverter CreateConverter(
        Type type,
        JsonSerializerOptions options) {
        Type[] typeArguments = type.GetGenericArguments();
        Type keyType = typeArguments[0];
        Type valueType = typeArguments[1];

        JsonConverter converter = (JsonConverter)Activator.CreateInstance(
            typeof(DictionaryConverterInner<,>).MakeGenericType(
                [keyType, valueType]),
            BindingFlags.Instance | BindingFlags.Public,
            binder: null,
            args: [options],
            culture: null)!;

        return converter;
    }

    private class DictionaryConverterInner<TKey, TValue> :
        JsonConverter<Dictionary<TKey, TValue>> {
        private readonly JsonConverter<TValue> _valueConverter;
        private readonly Type _keyType;
        private readonly Type _valueType;


        public DictionaryConverterInner(JsonSerializerOptions options) {
            // For performance, use the existing converter.
            _valueConverter = (JsonConverter<TValue>)options
                .GetConverter(typeof(TValue));

            // Cache the key and value types.
            _keyType = typeof(TKey);
            _valueType = typeof(TValue);
        }

        public override Dictionary<TKey, TValue> Read(
            ref Utf8JsonReader reader,
            Type typeToConvert,
            JsonSerializerOptions options) {
            if (reader.TokenType != JsonTokenType.StartObject) {
                throw new JsonException();
            }

            var dictionary = new Dictionary<TKey, TValue>();

            while (reader.Read()) {
                if (reader.TokenType == JsonTokenType.EndObject) {
                    return dictionary;
                }

                // Get the key.
                if (reader.TokenType != JsonTokenType.PropertyName) {
                    throw new JsonException();
                }

                string? propertyName = reader.GetString();
                TKey key = JsonSerializer.Deserialize<TKey>(propertyName, options);
                // For performance, parse with ignoreCase:false first.
                if (key == null) {
                    throw new JsonException(
                        $"Unable to parse \"{propertyName}\" to \"{_keyType}\".");
                }

                // Get the value.
                reader.Read();
                TValue value = _valueConverter.Read(ref reader, _valueType, options)!;

                // Add to dictionary.
                dictionary.Add(key, value);
            }

            throw new JsonException();
        }

        public override void Write(
            Utf8JsonWriter writer,
            Dictionary<TKey, TValue> dictionary,
            JsonSerializerOptions options) {
            writer.WriteStartObject();

            foreach ((TKey key, TValue value) in dictionary) {
                string propertyName = JsonSerializer.Serialize(key, options);
                writer.WritePropertyName(propertyName);

                _valueConverter.Write(writer, value, options);
            }

            writer.WriteEndObject();
        }
    }
}