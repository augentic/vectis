import Foundation
import Shared
import SharedTypes

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    private let core: CoreFfi

    init() {
        core = CoreFfi()
        view = try! .bincodeDeserialize(input: [UInt8](core.view()))
        update(.navigate(.todoList))
    }

    func update(_ event: Event) {
        let effects = [UInt8](
            core.update(data: Data(try! event.bincodeSerialize()))
        )
        let requests: [Request] = try! .bincodeDeserialize(input: effects)
        for request in requests {
            processEffect(request)
        }
    }

    func processEffect(_ request: Request) {
        switch request.effect {
        case .render:
            view = try! .bincodeDeserialize(input: [UInt8](core.view()))

        case let .http(httpRequest):
            Task {
                let response = await performHttpRequest(httpRequest)
                let effects = [UInt8](
                    core.resolve(
                        id: request.id,
                        data: Data(try! HttpResult.ok(response).bincodeSerialize())
                    )
                )
                let requests: [Request] = try! .bincodeDeserialize(input: effects)
                for request in requests {
                    processEffect(request)
                }
            }

        case let .keyValue(kvOp):
            Task {
                let result = performKeyValueOp(kvOp)
                let effects = [UInt8](
                    core.resolve(
                        id: request.id,
                        data: Data(try! result.bincodeSerialize())
                    )
                )
                let requests: [Request] = try! .bincodeDeserialize(input: effects)
                for request in requests {
                    processEffect(request)
                }
            }

        case let .time(timeRequest):
            Task {
                let response = await handleTimeRequest(timeRequest)
                let effects = [UInt8](
                    core.resolve(
                        id: request.id,
                        data: Data(try! response.bincodeSerialize())
                    )
                )
                let requests: [Request] = try! .bincodeDeserialize(input: effects)
                for request in requests {
                    processEffect(request)
                }
            }

        case let .serverSentEvents(sseRequest):
            Task {
                for await response in requestSse(sseRequest) {
                    let effects = [UInt8](
                        core.resolve(
                            id: request.id,
                            data: Data(try! response.bincodeSerialize())
                        )
                    )
                    let requests: [Request] = try! .bincodeDeserialize(input: effects)
                    for request in requests {
                        processEffect(request)
                    }
                }
            }
        }
    }

    // MARK: - HTTP

    private func performHttpRequest(_ request: HttpRequest) async -> HttpResponse {
        var urlRequest = URLRequest(url: URL(string: request.url)!)
        urlRequest.httpMethod = request.method

        for header in request.headers {
            urlRequest.setValue(header.value, forHTTPHeaderField: header.name)
        }

        if !request.body.isEmpty {
            urlRequest.httpBody = Data(request.body)
        }

        do {
            let (data, response) = try await URLSession.shared.data(for: urlRequest)
            let httpResponse = response as! HTTPURLResponse
            return HttpResponse(
                status: UInt16(httpResponse.statusCode),
                headers: httpResponse.allHeaderFields.map { key, value in
                    HttpHeader(
                        name: String(describing: key),
                        value: String(describing: value)
                    )
                },
                body: [UInt8](data)
            )
        } catch {
            return HttpResponse(status: 0, headers: [], body: [])
        }
    }

    // MARK: - Key-Value

    private func performKeyValueOp(_ op: KeyValueOperation) -> KeyValueResult {
        let defaults = UserDefaults.standard

        switch op {
        case let .get(key):
            if let data = defaults.data(forKey: key) {
                return .ok(response: .get(value: .bytes([UInt8](data))))
            }
            return .ok(response: .get(value: .none))

        case let .set(key, value):
            let previous: Value
            if let existing = defaults.data(forKey: key) {
                previous = .bytes([UInt8](existing))
            } else {
                previous = .none
            }
            defaults.set(Data(value), forKey: key)
            return .ok(response: .set(previous: previous))

        case let .delete(key):
            let previous: Value
            if let existing = defaults.data(forKey: key) {
                previous = .bytes([UInt8](existing))
            } else {
                previous = .none
            }
            defaults.removeObject(forKey: key)
            return .ok(response: .delete(previous: previous))

        case let .exists(key):
            return .ok(response: .exists(isPresent: defaults.object(forKey: key) != nil))

        case let .listKeys(prefix, _):
            let allKeys = defaults.dictionaryRepresentation().keys
            let filtered = allKeys.filter { $0.hasPrefix(prefix) }.sorted()
            return .ok(response: .listKeys(keys: filtered, nextCursor: 0))
        }
    }

    // MARK: - Time

    private func handleTimeRequest(_ request: TimeRequest) async -> TimeResponse {
        switch request {
        case .now:
            let now = Date()
            let secs = UInt64(now.timeIntervalSince1970)
            let nanos = UInt32((now.timeIntervalSince1970 - Double(secs)) * 1_000_000_000)
            return .now(instant: Instant(seconds: secs, nanos: nanos))

        case let .notifyAt(id, instant):
            let targetDate = Date(
                timeIntervalSince1970: Double(instant.seconds) + Double(instant.nanos) / 1_000_000_000
            )
            let delay = targetDate.timeIntervalSinceNow
            if delay > 0 {
                try? await Task.sleep(for: .seconds(delay))
            }
            return .instantArrived(id: id)

        case let .notifyAfter(id, duration):
            let delayNanos = Double(duration.nanos)
            let delaySecs = delayNanos / 1_000_000_000
            if delaySecs > 0 {
                try? await Task.sleep(for: .seconds(delaySecs))
            }
            return .durationElapsed(id: id)

        case let .clear(id):
            return .cleared(id: id)
        }
    }

    // MARK: - Server-Sent Events

    private func requestSse(_ request: SseRequest) -> AsyncStream<SseResponse> {
        AsyncStream { continuation in
            Task {
                guard let url = URL(string: request.url) else {
                    continuation.yield(.done)
                    continuation.finish()
                    return
                }

                do {
                    let (bytes, _) = try await URLSession.shared.bytes(from: url)
                    for try await line in bytes.lines {
                        let chunk = Array((line + "\n").utf8)
                        continuation.yield(.chunk(chunk))
                    }
                } catch {
                    // Connection closed or failed
                }
                continuation.yield(.done)
                continuation.finish()
            }
        }
    }
}
