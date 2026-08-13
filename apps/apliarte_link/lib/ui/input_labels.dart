/// Etiquetas de los comandos de cambio de control (compartidas entre
/// CommandSummary y SettingsDialog).
const Map<String, String> mouseCommandLabels = {
  'triple_right_click': 'Triple clic derecho',
  'triple_middle_click': 'Triple clic central',
  'disabled': 'Desactivado',
};

const Map<String, String> keyboardCommandLabels = {
  'triple_right_shift': 'Triple Shift derecho',
  'triple_escape': 'Triple Escape',
  'disabled': 'Desactivado',
};

const Map<String, String> edgeSideLabels = {
  'left': 'Borde izquierdo',
  'right': 'Borde derecho',
  'top': 'Borde superior',
  'bottom': 'Borde inferior',
};

/// Traduce un valor de configuración a su etiqueta legible.
String labelFor(Map<String, String> labels, String value) {
  return labels[value] ?? value;
}
