import QtQuick 2.15
import QtQuick.Layouts 1.15
import QtQuick.Controls 2.15
import org.kde.plasma.plasmoid 2.0
import org.kde.plasma.core 2.0 as PlasmaCore

PlasmoidItem {
    id: root

    preferredRepresentation: compactRepresentation
    toolTipMainText: topBarText || plasmoid.title
    toolTipSubText: prayerNextText || footerHolidaysText

    Plasmoid.title: "BOLOOT Persian Calendar"
    Plasmoid.icon: "preferences-system-time"

    property string topBarText: ""
    property string holidaysText: ""
    property string prayerNextText: ""
    property var monthView: null
    property int displayYear: 0
    property int displayMonth: 0
    property string selectedGregorian: ""
    property string selectedDayHolidaysText: ""

    readonly property bool isRtl: monthView && monthView.text_direction === "rtl"

    readonly property string footerHolidaysText: {
        if (selectedGregorian.length > 0) {
            if (selectedDayHolidaysText.length > 0)
                return "تعطیلات: " + selectedDayHolidaysText
            return ""
        }
        if (holidaysText.length > 0)
            return "تعطیلات: " + holidaysText
        return ""
    }

    function refresh() {
        previewSource.refresh()
        holidaysSource.refresh()
        prayerSource.refresh()
        monthSource.refresh()
    }

    function goToday() {
        displayYear = 0
        displayMonth = 0
        selectedGregorian = ""
        selectedDayHolidaysText = ""
        monthSource.refresh()
    }

    function changeMonth(delta) {
        if (!monthView) {
            monthSource.refresh()
            return
        }
        var month = displayMonth + delta
        var year = displayYear
        if (month < 1) {
            month = 12
            year -= 1
        } else if (month > 12) {
            month = 1
            year += 1
        }
        displayYear = year
        displayMonth = month
        monthSource.refresh()
    }

    function monthCommand() {
        if (displayYear > 0 && displayMonth > 0)
            return "boloot-calendar-ctl month --year " + displayYear + " --month " + displayMonth
        return "boloot-calendar-ctl month"
    }

    function updateSelectedHolidays() {
        if (!monthView || selectedGregorian.length === 0) {
            selectedDayHolidaysText = ""
            return
        }
        var cells = monthView.cells || []
        for (var i = 0; i < cells.length; i++) {
            var cell = cells[i]
            if (cell.gregorian_date === selectedGregorian) {
                if (cell.holiday_names && cell.holiday_names.length > 0)
                    selectedDayHolidaysText = cell.holiday_names.join("، ")
                else
                    selectedDayHolidaysText = ""
                return
            }
        }
        selectedDayHolidaysText = ""
    }

    function ensureDefaultSelection() {
        if (selectedGregorian.length > 0 || !monthView || !monthView.cells)
            return
        var cells = monthView.cells
        for (var i = 0; i < cells.length; i++) {
            var cell = cells[i]
            if (cell.is_today && cell.gregorian_date) {
                selectedGregorian = cell.gregorian_date
                break
            }
        }
        updateSelectedHolidays()
    }

    function selectDay(cell) {
        if (!cell || !cell.gregorian_date)
            return
        selectedGregorian = cell.gregorian_date
        if (!cell.is_current_month) {
            displayYear = cell.jalali_year
            displayMonth = cell.jalali_month
            monthSource.refresh()
            return
        }
        updateSelectedHolidays()
    }

    function cellColor(cell) {
        if (!cell)
            return PlasmaCore.Theme.textColor
        var appearance = monthView && monthView.appearance ? monthView.appearance : null
        if (appearance && !appearance.use_system_theme) {
            if (cell.is_today && appearance.today_color)
                return appearance.today_color
            if (cell.is_holiday && appearance.holiday_color)
                return appearance.holiday_color
        }
        if (cell.is_today)
            return PlasmaCore.Theme.highlightColor
        if (cell.is_holiday)
            return PlasmaCore.Theme.negativeTextColor
        return PlasmaCore.Theme.textColor
    }

    PlasmaCore.DataSource {
        id: previewSource
        engine: "executable"
        connectedSources: []
        onNewData: {
            if (data["exit code"] !== undefined && data["exit code"] !== "0")
                return
            var text = (data["stdout"] || "").trim()
            if (text.length > 0)
                root.topBarText = text
        }
        function refresh() {
            disconnectSource("preview")
            connectSource("boloot-calendar-ctl preview")
        }
    }

    PlasmaCore.DataSource {
        id: holidaysSource
        engine: "executable"
        connectedSources: []
        onNewData: {
            if (data["exit code"] !== undefined && data["exit code"] !== "0")
                return
            var raw = (data["stdout"] || "").trim()
            if (raw.length === 0) {
                root.holidaysText = ""
                return
            }
            try {
                var list = JSON.parse(raw)
                var names = []
                for (var i = 0; i < list.length; i++)
                    names.push(list[i].name)
                root.holidaysText = names.length > 0 ? names.join("، ") : ""
            } catch (e) {
                root.holidaysText = ""
            }
        }
        function refresh() {
            disconnectSource("holidays")
            connectSource("boloot-calendar-ctl holidays-today")
        }
    }

    PlasmaCore.DataSource {
        id: prayerSource
        engine: "executable"
        connectedSources: []
        onNewData: {
            if (data["exit code"] !== undefined && data["exit code"] !== "0")
                return
            var raw = (data["stdout"] || "").trim()
            var nextLine = ""
            var lines = raw.split("\n")
            for (var i = 0; i < lines.length; i++) {
                if (lines[i].indexOf("بعدی:") === 0)
                    nextLine = lines[i]
            }
            root.prayerNextText = nextLine
        }
        function refresh() {
            disconnectSource("prayer")
            connectSource("boloot-calendar-ctl prayer")
        }
    }

    PlasmaCore.DataSource {
        id: monthSource
        engine: "executable"
        connectedSources: []
        onNewData: {
            if (data["exit code"] !== undefined && data["exit code"] !== "0")
                return
            var raw = (data["stdout"] || "").trim()
            if (raw.length === 0)
                return
            try {
                var view = JSON.parse(raw)
                root.monthView = view
                root.displayYear = view.display_year !== undefined ? view.display_year : view.jalali_year
                root.displayMonth = view.display_month !== undefined ? view.display_month : view.jalali_month
                root.ensureDefaultSelection()
                root.updateSelectedHolidays()
            } catch (e) {
                root.monthView = null
            }
        }
        function refresh() {
            disconnectSource("month")
            connectSource(root.monthCommand())
        }
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 60000
        running: true
        repeat: true
        onTriggered: root.refresh()
    }

    compactRepresentation: Item {
        Layout.minimumWidth: compactLabel.implicitWidth + PlasmaCore.Units.smallSpacing * 2
        Layout.minimumHeight: compactLabel.implicitHeight + PlasmaCore.Units.smallSpacing

        Text {
            id: compactLabel
            anchors.centerIn: parent
            text: root.topBarText || "…"
            font.pointSize: 9
            color: PlasmaCore.Theme.textColor
        }
    }

    fullRepresentation: Item {
        Layout.minimumWidth: 320
        Layout.minimumHeight: 300

        LayoutMirroring.enabled: root.isRtl
        LayoutMirroring.childrenInherit: true

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: PlasmaCore.Units.smallSpacing
            spacing: PlasmaCore.Units.smallSpacing

            RowLayout {
                Layout.fillWidth: true

                ToolButton {
                    text: "›"
                    onClicked: root.changeMonth(-1)
                }

                Label {
                    text: monthView ? (monthView.title || monthView.month_name || "") : ""
                    font.bold: true
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    elide: Text.ElideRight
                }

                ToolButton {
                    text: "‹"
                    onClicked: root.changeMonth(1)
                }
            }

            ToolButton {
                text: "امروز"
                Layout.alignment: Qt.AlignHCenter
                onClicked: root.goToday()
            }

            GridLayout {
                Layout.fillWidth: true
                columns: 7
                rowSpacing: 2
                columnSpacing: 2
                visible: monthView !== null
                layoutDirection: root.isRtl ? Qt.RightToLeft : Qt.LeftToRight

                Repeater {
                    model: monthView ? monthView.weekday_headers : []
                    Label {
                        text: modelData
                        Layout.fillWidth: true
                        horizontalAlignment: Text.AlignHCenter
                        font.pointSize: 8
                        opacity: 0.75
                    }
                }

                Repeater {
                    model: monthView ? monthView.cells : []
                    Item {
                        Layout.preferredWidth: 36
                        Layout.preferredHeight: 40
                        opacity: modelData.is_current_month ? 1.0 : 0.4

                        Rectangle {
                            anchors.fill: parent
                            radius: 18
                            visible: modelData.is_today
                                    && (!monthView
                                        || !monthView.appearance
                                        || monthView.appearance.use_system_theme)
                            color: PlasmaCore.Theme.highlightColor
                            opacity: 0.25
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: 18
                            visible: monthView
                                    && monthView.appearance
                                    && !monthView.appearance.use_system_theme
                                    && (modelData.is_today || modelData.is_holiday)
                            color: modelData.is_today
                                ? (monthView.appearance.today_background_color || "transparent")
                                : (monthView.appearance.holiday_background_color || "transparent")
                            border.width: modelData.is_today && modelData.is_holiday ? 2 : 0
                            border.color: modelData.is_holiday && monthView.appearance.holiday_color
                                ? monthView.appearance.holiday_color
                                : "transparent"
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: 18
                            visible: modelData.gregorian_date
                                    && modelData.gregorian_date === root.selectedGregorian
                            color: "transparent"
                            border.width: 2
                            border.color: PlasmaCore.Theme.highlightColor
                        }

                        ColumnLayout {
                            anchors.centerIn: parent
                            spacing: 0

                            Label {
                                text: modelData.day_label || ""
                                horizontalAlignment: Text.AlignHCenter
                                Layout.alignment: Qt.AlignHCenter
                                font.bold: modelData.is_today
                                color: root.cellColor(modelData)
                            }

                            Label {
                                text: modelData.secondary_label || ""
                                visible: modelData.secondary_label
                                horizontalAlignment: Text.AlignHCenter
                                Layout.alignment: Qt.AlignHCenter
                                font.pointSize: 7
                                opacity: 0.7
                                color: PlasmaCore.Theme.textColor
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            hoverEnabled: true
                            ToolTip.visible: containsMouse && modelData.tooltip
                            ToolTip.text: modelData.tooltip || ""
                            onClicked: root.selectDay(modelData)
                        }
                    }
                }
            }

            Label {
                text: root.topBarText
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                visible: root.topBarText.length > 0
            }

            Label {
                text: root.prayerNextText
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                visible: root.prayerNextText.length > 0
                color: {
                    var appearance = monthView && monthView.appearance ? monthView.appearance : null
                    if (appearance && !appearance.use_system_theme && appearance.prayer_color)
                        return appearance.prayer_color
                    return PlasmaCore.Theme.positiveTextColor
                }
            }

            Label {
                text: root.footerHolidaysText
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                visible: root.footerHolidaysText.length > 0
            }

            Item { Layout.fillHeight: true }

            Label {
                text: "BOLOOT Persian Calendar — boloot.ir"
                font.pointSize: 8
                opacity: 0.65
                Layout.fillWidth: true
                horizontalAlignment: Text.AlignHCenter
            }
        }
    }
}
