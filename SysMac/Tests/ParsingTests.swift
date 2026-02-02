import XCTest
@testable import SysMac

final class ParsingTests: XCTestCase {

    // MARK: - ProcessService.parsePsLine

    func testParsePsLineValid() {
        let line = "  123     1  5.2  8192  0.3 root     S    2 /usr/sbin/syslogd"
        let proc = ProcessService.parsePsLine(line)
        XCTAssertNotNil(proc)
        XCTAssertEqual(proc?.pid, 123)
        XCTAssertEqual(proc?.ppid, 1)
        XCTAssertEqual(proc?.cpuUsage ?? 0, 5.2, accuracy: 0.01)
        XCTAssertEqual(proc?.memoryMB ?? 0, 8.0, accuracy: 0.01)  // 8192 KB / 1024
        XCTAssertEqual(proc?.memoryPercent ?? 0, 0.3, accuracy: 0.01)
        XCTAssertEqual(proc?.user, "root")
        XCTAssertEqual(proc?.state, "Suspendido")  // S state
        XCTAssertEqual(proc?.threads, 2)
        XCTAssertEqual(proc?.name, "syslogd")
    }

    func testParsePsLineRunning() {
        let line = "  456     1  99.0  2048  1.5 me       R    4 /usr/bin/top"
        let proc = ProcessService.parsePsLine(line)
        XCTAssertNotNil(proc)
        XCTAssertEqual(proc?.state, "Ejecutando")  // R state
    }

    func testParsePsLineZombie() {
        let line = "  789     1  0.0  0  0.0 nobody   Z    1 /some/zombie"
        let proc = ProcessService.parsePsLine(line)
        XCTAssertNotNil(proc)
        XCTAssertEqual(proc?.state, "Zombie")  // Z state
    }

    func testParsePsLineInvalid() {
        XCTAssertNil(ProcessService.parsePsLine(""))
        XCTAssertNil(ProcessService.parsePsLine("not enough fields"))
    }

    // MARK: - ProcessService.parseState

    func testParseState() {
        XCTAssertEqual(ProcessService.parseState("R"), "Ejecutando")
        XCTAssertEqual(ProcessService.parseState("S"), "Suspendido")
        XCTAssertEqual(ProcessService.parseState("I"), "Inactivo")
        XCTAssertEqual(ProcessService.parseState("U"), "Espera")
        XCTAssertEqual(ProcessService.parseState("Z"), "Zombie")
        XCTAssertEqual(ProcessService.parseState("T"), "Detenido")
        XCTAssertEqual(ProcessService.parseState("X"), "Desconocido")
        XCTAssertEqual(ProcessService.parseState(""), "Desconocido")
    }

    // MARK: - NetworkService.parseAddress

    func testParseAddressIPv4() {
        let (host, port) = NetworkService.parseAddress("192.168.1.1.443")
        XCTAssertEqual(host, "192.168.1.1")
        XCTAssertEqual(port, 443)
    }

    func testParseAddressIPv6() {
        let (host, port) = NetworkService.parseAddress("::1.8888")
        XCTAssertEqual(host, "::1")
        XCTAssertEqual(port, 8888)
    }

    func testParseAddressWildcard() {
        let (host, port) = NetworkService.parseAddress("*.*")
        XCTAssertEqual(host, "*")
        XCTAssertEqual(port, 0)
    }

    func testParseAddressLocalhost() {
        let (host, port) = NetworkService.parseAddress("127.0.0.1.80")
        XCTAssertEqual(host, "127.0.0.1")
        XCTAssertEqual(port, 80)
    }

    // MARK: - PortScannerService.detectServiceType

    func testDetectServiceTypeByName() {
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "node", port: 3000), "Node.js")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "Python3", port: 8000), "Python")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "java", port: 8080), "Java")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "postgres", port: 5432), "PostgreSQL")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "redis-server", port: 6379), "Redis")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "mongod", port: 27017), "MongoDB")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "nginx", port: 80), "Nginx")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "httpd", port: 80), "Apache")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "docker-proxy", port: 3000), "Docker")
    }

    func testDetectServiceTypeByPort() {
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "unknown", port: 80), "HTTP")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "unknown", port: 443), "HTTPS")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "unknown", port: 22), "SSH")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "unknown", port: 3000), "Dev Server")
        XCTAssertEqual(PortScannerService.detectServiceType(processName: "unknown", port: 9999), "Other")
    }

    // MARK: - DaemonsService.parseLaunchctlLine

    func testParseLaunchctlLineRunning() {
        let line = "12345\t0\tcom.example.daemon"
        let svc = DaemonsService.parseLaunchctlLine(line, kind: "User Agent")
        XCTAssertNotNil(svc)
        XCTAssertEqual(svc?.label, "com.example.daemon")
        XCTAssertEqual(svc?.pid, 12345)
        XCTAssertEqual(svc?.status, "running")
        XCTAssertEqual(svc?.kind, "User Agent")
    }

    func testParseLaunchctlLineStopped() {
        let line = "-\t0\tcom.example.stopped"
        let svc = DaemonsService.parseLaunchctlLine(line, kind: "User Agent")
        XCTAssertNotNil(svc)
        XCTAssertEqual(svc?.label, "com.example.stopped")
        XCTAssertNil(svc?.pid)
        XCTAssertEqual(svc?.status, "stopped")
    }

    func testParseLaunchctlLineError() {
        let line = "-\t78\tcom.example.errored"
        let svc = DaemonsService.parseLaunchctlLine(line, kind: "System Agent")
        XCTAssertNotNil(svc)
        XCTAssertEqual(svc?.status, "error")
        XCTAssertEqual(svc?.lastExitStatus, 78)
    }

    func testParseLaunchctlLineInvalid() {
        XCTAssertNil(DaemonsService.parseLaunchctlLine("", kind: ""))
        XCTAssertNil(DaemonsService.parseLaunchctlLine("too few", kind: ""))
    }
}
