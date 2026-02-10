# Contributing to SysMac

¡Gracias por tu interés en contribuir a SysMac! 🎉

Esta guía te ayudará a empezar con el proceso de contribución.

## 🤝 Cómo Contribuir

### Reportando Bugs

Antes de reportar un bug:

1. **Búsqueda**: Busca en [Issues existentes](https://github.com/DANO-AMP/MMAC/issues) para ver si ya ha sido reportado
2. **Verifica la versión**: Asegúrate de estar usando la última versión
3. **Reproduce**: Intenta reproducir el problema con pasos claros

Al crear un issue, incluye:

- **Título**: Descriptivo y específico
- **Descripción**: Qué sucedió y qué esperabas que sucediera
- **Pasos**: Pasos claros para reproducir el problema
- **Environment**:
  - Versión de SysMac
  - Versión de macOS
  - Arquitectura (Intel/Apple Silicon)
  - Capturas de pantalla si es aplicable

### Sugerencias

¡Nos encantan las sugerencias! Al crear una sugerencia:

- **Usa el formato**: `[Feature] Título de la sugerencia`
- **Describe el caso de uso**: ¿Por qué necesitas esto?
- **Propón una solución**: ¿Cómo imaginas que funcionaría?

## 🛠️ Código de Contribución

### Configuración del Entorno

```bash
# Clona el repositorio
git clone https://github.com/DANO-AMP/MMAC.git
cd MMAC

# Añade el repositorio original como upstream
git remote add upstream https://github.com/DANO-AMP/MMAC.git
```

### Creando una Rama

```bash
# Crea una nueva rama para tu feature
git checkout -b feature/tu-nombre-de-feature

# O para un fix
git checkout -b fix/tu-nombre-de-fix
```

### Código Style

Sigue estas guías:

- **Swift**: Usa [Swift Style Guide](https://github.com/github/swift-style-guide)
- **Nombre de variables**: camelCase
- **Nombre de constantes**: camelCase
- **Indentación**: 4 espacios
- **Líneas**: Máximo 100 caracteres
- **Comentarios**: Explica el "por qué", no el "qué"

### Git Commits

Usa [Conventional Commits](https://www.conventionalcommits.org/):

```
<tipo>(<ámbito>): <descripción>

[opcional cuerpo]

[opcional pie]
```

**Tipos comunes**:

- `feat`: Nueva funcionalidad
- `fix`: Corrección de bug
- `docs`: Cambios en documentación
- `style`: Formato, missing semicolons, etc.
- `refactor`: Cambios en el código
- `perf`: Mejoras de rendimiento
- `test`: Añadiendo o actualizando tests
- `chore`: Actualización de tareas de build/config

**Ejemplos**:

```
feat(memory): add memory optimization button
fix(bluetooth): crash when scanning devices
docs(readme): update installation instructions
style(swift): format code with swiftformat
refactor(analyzer): simplify algorithm
```

### Pull Requests

1. **Actualiza tu rama**:

```bash
git fetch upstream
git rebase upstream/main
```

2. **Push a tu fork**:

```bash
git push origin feature/tu-feature
```

3. **Crea el Pull Request** en GitHub

**Título del PR**: Siguiendo el formato de commits

**Descripción del PR**:
- ¿Qué hace este PR?
- ¿Por qué es necesario?
- ¿Cómo has probado los cambios?
- ¿Hay breaking changes?
- Screenshots si es una mejora visual

## 🧪 Testing

### Ejecutar Tests

```bash
# Ejecutar todos los tests
swift test

# Ejecutar tests específicos
swift test --filter SysMacTests
```

### Manual Testing

Antes de crear un PR, asegúrate de:

1. [ ] La aplicación compila sin errores
2. [ ] La funcionalidad nueva funciona como esperado
3. [ ] No rompe funcionalidades existentes
4. [ ] Funciona en diferentes tamaños de ventana
5. [ ] Funciona en modo oscuro
6. [ ] No hay memory leaks

## 📝 Documentación

### Código

- Documenta las clases y funciones públicas
- Usa comments para explicar lógica compleja
- Actualiza README.md si añades nuevas features

### Changelog

Actualiza el CHANGELOG.md cuando:
- Añades nuevas features
- Corriges bugs importantes
- Haces breaking changes

## 🎨 Diseño

### Guidelines

- **Consistencia**: Mantén el estilo visual consistente
- **Responsive**: Adaptable a diferentes tamaños
- **Accesibilidad**: Usa etiquetas y colores accesibles
- **Performance**: Evita animaciones pesadas

### SwiftUI

- Usa `LazyVStack` para listas grandes
- Usa `.environmentObject` para datos compartidos
- Usa `.sheet` para modales
- Usa `.alert` para alertas críticas

## 🐛 Debugging

### Logs

```swift
import os
os_log(OSLog.default, "Debug message: %@", variable)
```

### Console.app

1. Abre Console.app
2. Filtra por "SysMac"
3. Busca errores o warnings

## 📞 Comunicación

- **Discusiones**: [GitHub Discussions](https://github.com/DANO-AMP/MMAC/discussions)
- **Issues**: [GitHub Issues](https://github.com/DANO-AMP/MMAC/issues)
- **Email**: [Contacto](mailto:contact@example.com)

## 🎓 Recursos

- [Swift Documentation](https://docs.swift.org/)
- [SwiftUI Documentation](https://developer.apple.com/documentation/swiftui)
- [macOS APIs](https://developer.apple.com/documentation/macos)

## 📄 Código de Conducta

Sé respetuoso y constructivo:

- Usa lenguaje inclusivo
- Acepta y da feedback constructivamente
- Sé paciente con nuevas contribuciones
- Respeta diferentes opiniones y experiencias

## ✨ Reconocimiento

Los contribuidores serán reconocidos en:

- README.md (sección Contributors)
- Releases notes
- Special thanks en milestones

---

¡Gracias por contribuir a SysMac! 🚀