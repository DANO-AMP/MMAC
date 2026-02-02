import XCTest
@testable import SysMac

final class FormattersTests: XCTestCase {

    // MARK: - formatSize

    func testFormatSizeZero() {
        XCTAssertEqual(Formatters.formatSize(UInt64(0)), "0 B")
    }

    func testFormatSizeBytes() {
        XCTAssertEqual(Formatters.formatSize(UInt64(500)), "500 B")
    }

    func testFormatSizeKilobytes() {
        XCTAssertEqual(Formatters.formatSize(UInt64(1024)), "1 KB")
    }

    func testFormatSizeMegabytes() {
        XCTAssertEqual(Formatters.formatSize(UInt64(1_048_576)), "1 MB")
    }

    func testFormatSizeGigabytes() {
        XCTAssertEqual(Formatters.formatSize(UInt64(1_073_741_824)), "1 GB")
    }

    func testFormatSizeTerabytes() {
        XCTAssertEqual(Formatters.formatSize(UInt64(1_099_511_627_776)), "1 TB")
    }

    func testFormatSizeFractional() {
        // 1.5 GB = 1,610,612,736 bytes
        XCTAssertEqual(Formatters.formatSize(UInt64(1_610_612_736)), "1.5 GB")
    }

    func testFormatSizeInt64() {
        XCTAssertEqual(Formatters.formatSize(Int64(0)), "0 B")
        XCTAssertEqual(Formatters.formatSize(Int64(1024)), "1 KB")
    }

    // MARK: - formatSpeed

    func testFormatSpeedZero() {
        XCTAssertEqual(Formatters.formatSpeed(0), "0 B/s")
    }

    func testFormatSpeedBytesPerSec() {
        XCTAssertEqual(Formatters.formatSpeed(512), "512 B/s")
    }

    func testFormatSpeedKBPerSec() {
        XCTAssertEqual(Formatters.formatSpeed(1024), "1 KB/s")
    }

    func testFormatSpeedMBPerSec() {
        XCTAssertEqual(Formatters.formatSpeed(1_048_576), "1 MB/s")
    }

    // MARK: - formatPercentage

    func testFormatPercentage() {
        XCTAssertEqual(Formatters.formatPercentage(50.0), "50.0%")
        XCTAssertEqual(Formatters.formatPercentage(99.9), "99.9%")
        XCTAssertEqual(Formatters.formatPercentage(0.0), "0.0%")
    }

    func testFormatPercentageNaN() {
        XCTAssertEqual(Formatters.formatPercentage(Double.nan), "0%")
    }

    func testFormatPercentageCustomDecimals() {
        XCTAssertEqual(Formatters.formatPercentage(33.333, decimals: 2), "33.33%")
    }

    // MARK: - formatTemperature

    func testFormatTemperature() {
        XCTAssertEqual(Formatters.formatTemperature(36.5), "36.5\u{00B0}C")
    }

    func testFormatTemperatureNaN() {
        XCTAssertEqual(Formatters.formatTemperature(Double.nan), "N/A")
    }

    // MARK: - formatDuration

    func testFormatDuration() {
        XCTAssertEqual(Formatters.formatDuration(hours: 2, minutes: 5), "2:05")
        XCTAssertEqual(Formatters.formatDuration(hours: 0, minutes: 0), "0:00")
        XCTAssertEqual(Formatters.formatDuration(hours: 10, minutes: 30), "10:30")
    }
}
