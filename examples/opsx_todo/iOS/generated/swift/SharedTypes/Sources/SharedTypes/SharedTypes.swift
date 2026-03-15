import Serde

func serializeArray<T, S: Serializer>(
    value: [T],
    serializer: S,
    serializeElement: (T, S) throws -> Void
) throws {
    try serializer.serialize_len(value: value.count)
    for item in value {
        try serializeElement(item, serializer)
    }
}

func deserializeArray<T, D: Deserializer>(
    deserializer: D,
    deserializeElement: (D) throws -> T
) throws -> [T] {
    let length = try deserializer.deserialize_len()
    var obj: [T] = []
    for _ in 0..<length {
        obj.append(try deserializeElement(deserializer))
    }
    return obj
}

indirect public enum Effect: Hashable {
    case render(RenderOperation)
    case http(HttpRequest)
    case keyValue(KeyValueOperation)
    case serverSentEvents(SseRequest)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .render(let x):
            try serializer.serialize_variant_index(value: 0)
            try x.serialize(serializer: serializer)
        case .http(let x):
            try serializer.serialize_variant_index(value: 1)
            try x.serialize(serializer: serializer)
        case .keyValue(let x):
            try serializer.serialize_variant_index(value: 2)
            try x.serialize(serializer: serializer)
        case .serverSentEvents(let x):
            try serializer.serialize_variant_index(value: 3)
            try x.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Effect {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try RenderOperation.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .render(x)
        case 1:
            let x = try HttpRequest.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .http(x)
        case 2:
            let x = try KeyValueOperation.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .keyValue(x)
        case 3:
            let x = try SseRequest.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .serverSentEvents(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Effect: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Effect {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct ErrorView: Hashable {
    @Indirect public var message: String
    @Indirect public var canRetry: Bool

    public init(message: String, canRetry: Bool) {
        self.message = message
        self.canRetry = canRetry
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.message)
        try serializer.serialize_bool(value: self.canRetry)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ErrorView {
        try deserializer.increase_container_depth()
        let message = try deserializer.deserialize_str()
        let canRetry = try deserializer.deserialize_bool()
        try deserializer.decrease_container_depth()
        return ErrorView(message: message, canRetry: canRetry)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ErrorView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Event: Hashable {
    case initialize
    case navigate(Route)
    case setInput(String)
    case addTodo(String, String)
    case editTitle(String, String, String)
    case toggleCompleted(String, String)
    case deleteTodo(String, String)
    case clearCompleted(String)
    case setFilter(Filter)
    case retrySync
    case connectSse
    case sseDisconnected

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .initialize:
            try serializer.serialize_variant_index(value: 0)
        case .navigate(let x):
            try serializer.serialize_variant_index(value: 1)
            try x.serialize(serializer: serializer)
        case .setInput(let x):
            try serializer.serialize_variant_index(value: 2)
            try serializer.serialize_str(value: x)
        case .addTodo(let x0, let x1):
            try serializer.serialize_variant_index(value: 3)
            try serializer.serialize_str(value: x0)
            try serializer.serialize_str(value: x1)
        case .editTitle(let x0, let x1, let x2):
            try serializer.serialize_variant_index(value: 4)
            try serializer.serialize_str(value: x0)
            try serializer.serialize_str(value: x1)
            try serializer.serialize_str(value: x2)
        case .toggleCompleted(let x0, let x1):
            try serializer.serialize_variant_index(value: 5)
            try serializer.serialize_str(value: x0)
            try serializer.serialize_str(value: x1)
        case .deleteTodo(let x0, let x1):
            try serializer.serialize_variant_index(value: 6)
            try serializer.serialize_str(value: x0)
            try serializer.serialize_str(value: x1)
        case .clearCompleted(let x):
            try serializer.serialize_variant_index(value: 7)
            try serializer.serialize_str(value: x)
        case .setFilter(let x):
            try serializer.serialize_variant_index(value: 8)
            try x.serialize(serializer: serializer)
        case .retrySync:
            try serializer.serialize_variant_index(value: 9)
        case .connectSse:
            try serializer.serialize_variant_index(value: 10)
        case .sseDisconnected:
            try serializer.serialize_variant_index(value: 11)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Event {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .initialize
        case 1:
            let x = try Route.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .navigate(x)
        case 2:
            let x = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .setInput(x)
        case 3:
            let x0 = try deserializer.deserialize_str()
            let x1 = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .addTodo(x0, x1)
        case 4:
            let x0 = try deserializer.deserialize_str()
            let x1 = try deserializer.deserialize_str()
            let x2 = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .editTitle(x0, x1, x2)
        case 5:
            let x0 = try deserializer.deserialize_str()
            let x1 = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .toggleCompleted(x0, x1)
        case 6:
            let x0 = try deserializer.deserialize_str()
            let x1 = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .deleteTodo(x0, x1)
        case 7:
            let x = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .clearCompleted(x)
        case 8:
            let x = try Filter.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .setFilter(x)
        case 9:
            try deserializer.decrease_container_depth()
            return .retrySync
        case 10:
            try deserializer.decrease_container_depth()
            return .connectSse
        case 11:
            try deserializer.decrease_container_depth()
            return .sseDisconnected
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Event: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Event {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Filter: Hashable {
    case all
    case active
    case completed

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .all:
            try serializer.serialize_variant_index(value: 0)
        case .active:
            try serializer.serialize_variant_index(value: 1)
        case .completed:
            try serializer.serialize_variant_index(value: 2)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Filter {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .all
        case 1:
            try deserializer.decrease_container_depth()
            return .active
        case 2:
            try deserializer.decrease_container_depth()
            return .completed
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Filter: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Filter {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum HttpError: Hashable {
    case url(String)
    case io(String)
    case timeout

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .url(let x):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_str(value: x)
        case .io(let x):
            try serializer.serialize_variant_index(value: 1)
            try serializer.serialize_str(value: x)
        case .timeout:
            try serializer.serialize_variant_index(value: 2)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpError {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .url(x)
        case 1:
            let x = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .io(x)
        case 2:
            try deserializer.decrease_container_depth()
            return .timeout
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for HttpError: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpError {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct HttpHeader: Hashable {
    @Indirect public var name: String
    @Indirect public var value: String

    public init(name: String, value: String) {
        self.name = name
        self.value = value
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.name)
        try serializer.serialize_str(value: self.value)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpHeader {
        try deserializer.increase_container_depth()
        let name = try deserializer.deserialize_str()
        let value = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return HttpHeader(name: name, value: value)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpHeader {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct HttpRequest: Hashable {
    @Indirect public var method: String
    @Indirect public var url: String
    @Indirect public var headers: [HttpHeader]
    @Indirect public var body: [UInt8]

    public init(method: String, url: String, headers: [HttpHeader], body: [UInt8]) {
        self.method = method
        self.url = url
        self.headers = headers
        self.body = body
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.method)
        try serializer.serialize_str(value: self.url)
        try serializeArray(value: self.headers, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_bytes(value: self.body)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpRequest {
        try deserializer.increase_container_depth()
        let method = try deserializer.deserialize_str()
        let url = try deserializer.deserialize_str()
        let headers = try deserializeArray(deserializer: deserializer) { deserializer in
            try HttpHeader.deserialize(deserializer: deserializer)
        }
        let body = try deserializer.deserialize_bytes()
        try deserializer.decrease_container_depth()
        return HttpRequest(method: method, url: url, headers: headers, body: body)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpRequest {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct HttpResponse: Hashable {
    @Indirect public var status: UInt16
    @Indirect public var headers: [HttpHeader]
    @Indirect public var body: [UInt8]

    public init(status: UInt16, headers: [HttpHeader], body: [UInt8]) {
        self.status = status
        self.headers = headers
        self.body = body
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u16(value: self.status)
        try serializeArray(value: self.headers, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_bytes(value: self.body)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpResponse {
        try deserializer.increase_container_depth()
        let status = try deserializer.deserialize_u16()
        let headers = try deserializeArray(deserializer: deserializer) { deserializer in
            try HttpHeader.deserialize(deserializer: deserializer)
        }
        let body = try deserializer.deserialize_bytes()
        try deserializer.decrease_container_depth()
        return HttpResponse(status: status, headers: headers, body: body)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpResponse {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum HttpResult: Hashable {
    case ok(HttpResponse)
    case err(HttpError)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .ok(let x):
            try serializer.serialize_variant_index(value: 0)
            try x.serialize(serializer: serializer)
        case .err(let x):
            try serializer.serialize_variant_index(value: 1)
            try x.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> HttpResult {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try HttpResponse.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .ok(x)
        case 1:
            let x = try HttpError.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .err(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for HttpResult: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> HttpResult {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum KeyValueError: Hashable {
    case io(message: String)
    case timeout
    case cursorNotFound
    case other(message: String)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .io(let message):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_str(value: message)
        case .timeout:
            try serializer.serialize_variant_index(value: 1)
        case .cursorNotFound:
            try serializer.serialize_variant_index(value: 2)
        case .other(let message):
            try serializer.serialize_variant_index(value: 3)
            try serializer.serialize_str(value: message)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> KeyValueError {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let message = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .io(message: message)
        case 1:
            try deserializer.decrease_container_depth()
            return .timeout
        case 2:
            try deserializer.decrease_container_depth()
            return .cursorNotFound
        case 3:
            let message = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .other(message: message)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for KeyValueError: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> KeyValueError {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum KeyValueOperation: Hashable {
    case get(key: String)
    case set(key: String, value: [UInt8])
    case delete(key: String)
    case exists(key: String)
    case listKeys(prefix: String, cursor: UInt64)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .get(let key):
            try serializer.serialize_variant_index(value: 0)
            try serializer.serialize_str(value: key)
        case .set(let key, let value):
            try serializer.serialize_variant_index(value: 1)
            try serializer.serialize_str(value: key)
            try serializeArray(value: value, serializer: serializer) { item, serializer in
                try serializer.serialize_u8(value: item)
            }
        case .delete(let key):
            try serializer.serialize_variant_index(value: 2)
            try serializer.serialize_str(value: key)
        case .exists(let key):
            try serializer.serialize_variant_index(value: 3)
            try serializer.serialize_str(value: key)
        case .listKeys(let prefix, let cursor):
            try serializer.serialize_variant_index(value: 4)
            try serializer.serialize_str(value: prefix)
            try serializer.serialize_u64(value: cursor)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> KeyValueOperation {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let key = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .get(key: key)
        case 1:
            let key = try deserializer.deserialize_str()
            let value = try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_u8()
            }
            try deserializer.decrease_container_depth()
            return .set(key: key, value: value)
        case 2:
            let key = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .delete(key: key)
        case 3:
            let key = try deserializer.deserialize_str()
            try deserializer.decrease_container_depth()
            return .exists(key: key)
        case 4:
            let prefix = try deserializer.deserialize_str()
            let cursor = try deserializer.deserialize_u64()
            try deserializer.decrease_container_depth()
            return .listKeys(prefix: prefix, cursor: cursor)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for KeyValueOperation: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> KeyValueOperation {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum KeyValueResponse: Hashable {
    case get(value: Value)
    case set(previous: Value)
    case delete(previous: Value)
    case exists(isPresent: Bool)
    case listKeys(keys: [String], nextCursor: UInt64)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .get(let value):
            try serializer.serialize_variant_index(value: 0)
            try value.serialize(serializer: serializer)
        case .set(let previous):
            try serializer.serialize_variant_index(value: 1)
            try previous.serialize(serializer: serializer)
        case .delete(let previous):
            try serializer.serialize_variant_index(value: 2)
            try previous.serialize(serializer: serializer)
        case .exists(let isPresent):
            try serializer.serialize_variant_index(value: 3)
            try serializer.serialize_bool(value: isPresent)
        case .listKeys(let keys, let nextCursor):
            try serializer.serialize_variant_index(value: 4)
            try serializeArray(value: keys, serializer: serializer) { item, serializer in
                try serializer.serialize_str(value: item)
            }
            try serializer.serialize_u64(value: nextCursor)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> KeyValueResponse {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let value = try Value.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .get(value: value)
        case 1:
            let previous = try Value.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .set(previous: previous)
        case 2:
            let previous = try Value.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .delete(previous: previous)
        case 3:
            let isPresent = try deserializer.deserialize_bool()
            try deserializer.decrease_container_depth()
            return .exists(isPresent: isPresent)
        case 4:
            let keys = try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_str()
            }
            let nextCursor = try deserializer.deserialize_u64()
            try deserializer.decrease_container_depth()
            return .listKeys(keys: keys, nextCursor: nextCursor)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for KeyValueResponse: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> KeyValueResponse {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum KeyValueResult: Hashable {
    case ok(response: KeyValueResponse)
    case err(error: KeyValueError)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .ok(let response):
            try serializer.serialize_variant_index(value: 0)
            try response.serialize(serializer: serializer)
        case .err(let error):
            try serializer.serialize_variant_index(value: 1)
            try error.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> KeyValueResult {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let response = try KeyValueResponse.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .ok(response: response)
        case 1:
            let error = try KeyValueError.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .err(error: error)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for KeyValueResult: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> KeyValueResult {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct RenderOperation: Hashable {

    public init() {
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> RenderOperation {
        try deserializer.increase_container_depth()
        try deserializer.decrease_container_depth()
        return RenderOperation()
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> RenderOperation {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct Request: Hashable {
    @Indirect public var id: UInt32
    @Indirect public var effect: Effect

    public init(id: UInt32, effect: Effect) {
        self.id = id
        self.effect = effect
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_u32(value: self.id)
        try self.effect.serialize(serializer: serializer)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Request {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_u32()
        let effect = try Effect.deserialize(deserializer: deserializer)
        try deserializer.decrease_container_depth()
        return Request(id: id, effect: effect)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Request {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Route: Hashable {
    case todoList

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .todoList:
            try serializer.serialize_variant_index(value: 0)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Route {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .todoList
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Route: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Route {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct SseRequest: Hashable {
    @Indirect public var url: String

    public init(url: String) {
        self.url = url
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.url)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> SseRequest {
        try deserializer.increase_container_depth()
        let url = try deserializer.deserialize_str()
        try deserializer.decrease_container_depth()
        return SseRequest(url: url)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> SseRequest {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum SseResponse: Hashable {
    case chunk([UInt8])
    case done

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .chunk(let x):
            try serializer.serialize_variant_index(value: 0)
            try serializeArray(value: x, serializer: serializer) { item, serializer in
                try serializer.serialize_u8(value: item)
            }
        case .done:
            try serializer.serialize_variant_index(value: 1)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> SseResponse {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            let x = try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_u8()
            }
            try deserializer.decrease_container_depth()
            return .chunk(x)
        case 1:
            try deserializer.decrease_container_depth()
            return .done
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for SseResponse: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> SseResponse {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct TodoItemView: Hashable {
    @Indirect public var id: String
    @Indirect public var title: String
    @Indirect public var completed: Bool

    public init(id: String, title: String, completed: Bool) {
        self.id = id
        self.title = title
        self.completed = completed
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializer.serialize_str(value: self.id)
        try serializer.serialize_str(value: self.title)
        try serializer.serialize_bool(value: self.completed)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> TodoItemView {
        try deserializer.increase_container_depth()
        let id = try deserializer.deserialize_str()
        let title = try deserializer.deserialize_str()
        let completed = try deserializer.deserialize_bool()
        try deserializer.decrease_container_depth()
        return TodoItemView(id: id, title: title, completed: completed)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> TodoItemView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

public struct TodoListView: Hashable {
    @Indirect public var items: [TodoItemView]
    @Indirect public var inputText: String
    @Indirect public var activeCount: String
    @Indirect public var pendingCount: String
    @Indirect public var syncStatus: String
    @Indirect public var filter: Filter
    @Indirect public var showClearCompleted: Bool

    public init(items: [TodoItemView], inputText: String, activeCount: String, pendingCount: String, syncStatus: String, filter: Filter, showClearCompleted: Bool) {
        self.items = items
        self.inputText = inputText
        self.activeCount = activeCount
        self.pendingCount = pendingCount
        self.syncStatus = syncStatus
        self.filter = filter
        self.showClearCompleted = showClearCompleted
    }

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        try serializeArray(value: self.items, serializer: serializer) { item, serializer in
            try item.serialize(serializer: serializer)
        }
        try serializer.serialize_str(value: self.inputText)
        try serializer.serialize_str(value: self.activeCount)
        try serializer.serialize_str(value: self.pendingCount)
        try serializer.serialize_str(value: self.syncStatus)
        try self.filter.serialize(serializer: serializer)
        try serializer.serialize_bool(value: self.showClearCompleted)
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> TodoListView {
        try deserializer.increase_container_depth()
        let items = try deserializeArray(deserializer: deserializer) { deserializer in
            try TodoItemView.deserialize(deserializer: deserializer)
        }
        let inputText = try deserializer.deserialize_str()
        let activeCount = try deserializer.deserialize_str()
        let pendingCount = try deserializer.deserialize_str()
        let syncStatus = try deserializer.deserialize_str()
        let filter = try Filter.deserialize(deserializer: deserializer)
        let showClearCompleted = try deserializer.deserialize_bool()
        try deserializer.decrease_container_depth()
        return TodoListView(items: items, inputText: inputText, activeCount: activeCount, pendingCount: pendingCount, syncStatus: syncStatus, filter: filter, showClearCompleted: showClearCompleted)
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> TodoListView {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum Value: Hashable {
    case none
    case bytes([UInt8])

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .none:
            try serializer.serialize_variant_index(value: 0)
        case .bytes(let x):
            try serializer.serialize_variant_index(value: 1)
            try serializeArray(value: x, serializer: serializer) { item, serializer in
                try serializer.serialize_u8(value: item)
            }
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> Value {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .none
        case 1:
            let x = try deserializeArray(deserializer: deserializer) { deserializer in
                try deserializer.deserialize_u8()
            }
            try deserializer.decrease_container_depth()
            return .bytes(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for Value: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> Value {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}

indirect public enum ViewModel: Hashable {
    case loading
    case todoList(TodoListView)
    case error(ErrorView)

    public func serialize<S: Serializer>(serializer: S) throws {
        try serializer.increase_container_depth()
        switch self {
        case .loading:
            try serializer.serialize_variant_index(value: 0)
        case .todoList(let x):
            try serializer.serialize_variant_index(value: 1)
            try x.serialize(serializer: serializer)
        case .error(let x):
            try serializer.serialize_variant_index(value: 2)
            try x.serialize(serializer: serializer)
        }
        try serializer.decrease_container_depth()
    }

    public func bincodeSerialize() throws -> [UInt8] {
        let serializer = BincodeSerializer.init();
        try self.serialize(serializer: serializer)
        return serializer.get_bytes()
    }

    public static func deserialize<D: Deserializer>(deserializer: D) throws -> ViewModel {
        let index = try deserializer.deserialize_variant_index()
        try deserializer.increase_container_depth()
        switch index {
        case 0:
            try deserializer.decrease_container_depth()
            return .loading
        case 1:
            let x = try TodoListView.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .todoList(x)
        case 2:
            let x = try ErrorView.deserialize(deserializer: deserializer)
            try deserializer.decrease_container_depth()
            return .error(x)
        default: throw DeserializationError.invalidInput(issue: "Unknown variant index for ViewModel: \(index)")
        }
    }

    public static func bincodeDeserialize(input: [UInt8]) throws -> ViewModel {
        let deserializer = BincodeDeserializer.init(input: input);
        let obj = try deserialize(deserializer: deserializer)
        if deserializer.get_buffer_offset() < input.count {
            throw DeserializationError.invalidInput(issue: "Some input bytes were not read")
        }
        return obj
    }
}
