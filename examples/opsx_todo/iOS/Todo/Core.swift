import Foundation
import Shared
import SharedTypes

@MainActor
class Core: ObservableObject {
    @Published var view: ViewModel

    private let core: CoreFfi
    private var sseTask: Task<Void, Never>?

    init() {
        core = CoreFfi()
        view = try! .bincodeDeserialize(input: [UInt8](core.view()))
        update(.initialize)
        update(.connectSse)
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
                let output = performKeyValueOp(kvOp)
                let effects = [UInt8](
                    core.resolve(
                        id: request.id,
                        data: Data(try! output.bincodeSerialize())
                    )
                )
                let requests: [Request] = try! .bincodeDeserialize(input: effects)
                for request in requests {
                    processEffect(request)
                }
            }

        case let .serverSentEvents(sseRequest):
            handleSseEffect(request: request, sseRequest: sseRequest)
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
        switch op {
        case let .get(key):
            if let data = UserDefaults.standard.data(forKey: key) {
                return .ok(response: .get(value: .bytes([UInt8](data))))
            }
            return .ok(response: .get(value: .none))

        case let .set(key, value):
            let previous = UserDefaults.standard.data(forKey: key)
            UserDefaults.standard.set(Data(value), forKey: key)
            if let prev = previous {
                return .ok(response: .set(previous: .bytes([UInt8](prev))))
            }
            return .ok(response: .set(previous: .none))

        case let .delete(key):
            let previous = UserDefaults.standard.data(forKey: key)
            UserDefaults.standard.removeObject(forKey: key)
            if let prev = previous {
                return .ok(response: .delete(previous: .bytes([UInt8](prev))))
            }
            return .ok(response: .delete(previous: .none))

        case let .exists(key):
            let isPresent = UserDefaults.standard.object(forKey: key) != nil
            return .ok(response: .exists(isPresent: isPresent))

        case let .listKeys(prefix, cursor):
            let allKeys = UserDefaults.standard.dictionaryRepresentation().keys
            let matching = allKeys.filter { $0.hasPrefix(prefix) }.sorted()
            return .ok(response: .listKeys(keys: matching, nextCursor: 0))
        }
    }

    // MARK: - Server-Sent Events

    private func handleSseEffect(request: Request, sseRequest: SseRequest) {
        sseTask?.cancel()
        sseTask = Task {
            guard let url = URL(string: sseRequest.url) else { return }
            do {
                let (bytes, _) = try await URLSession.shared.bytes(from: url)
                for try await line in bytes.lines {
                    if Task.isCancelled { break }
                    let chunk = [UInt8]((line + "\n").utf8)
                    let response = SseResponse.chunk(chunk)
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
            } catch {
                if !Task.isCancelled {
                    update(.sseDisconnected)
                }
            }
            if !Task.isCancelled {
                let done = SseResponse.done
                let effects = [UInt8](
                    core.resolve(
                        id: request.id,
                        data: Data(try! done.bincodeSerialize())
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
