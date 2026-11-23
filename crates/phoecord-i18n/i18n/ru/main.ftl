# Общие строки PhoeCord (русский).
# Глоссарий (используем везде одинаково):
#   plugin — плагин; proxy — прокси; config — конфиг (в справке и CLI), «файл конфигурации» в длинных фразах;
#   CA — сертификат CA (локальный); certificate authority — центр сертификации;
#   trust store — хранилище доверенных сертификатов; gateway — шлюз; selftest — самопроверка;
#   enable / disable — включить / выключить; launch — запустить; autostart — автозапуск.
#   install.log, config.toml, имена команд, флагов и id плагинов не переводятся.

plugin-enabled = Плагин { $name } включён.
plugin-disabled = Плагин { $name } отключён.
plugin-unknown = Плагин «{ $name }» не найден.
plugin-failed = Плагин { $name } завершился с ошибкой и был отключён: { $reason }

proxy-listening = Прокси слушает на { $addr }.
proxy-stopped = Прокси остановлен.
proxy-port-busy = Порт { $port } уже занят.

ca-created = Центр сертификации создан: { $path }.
ca-exists = Центр сертификации уже существует: { $path }.
ca-trusted = Центр сертификации добавлен в хранилище доверенных сертификатов.
ca-trust-failed = Не удалось добавить центр сертификации в хранилище доверенных сертификатов: { $reason }

error-generic = Что-то пошло не так: { $reason }
error-config-invalid = Файл конфигурации { $path } недействителен: { $reason }
error-permission-denied = Доступ запрещён. Попробуйте запустить от имени администратора.

lang-changed = Язык изменён на { $language }.

# CLI: setup, uninstall, paths
cli-setup-done = Готово. Полностью закройте Discord и запустите его командой `phoecord launch`.
cli-uninstall-done = PhoeCord удалён. Перезапустите Discord без `phoecord launch`.
cli-uninstall-incomplete = не все шаги удалось отменить; они остались в install.log, запустите `phoecord uninstall` ещё раз
cli-setup-steps = Новых шагов установки записано в install.log: { $count }
cli-uninstall-removed = удалено   { $what }
cli-uninstall-kept = оставлено { $what }
cli-uninstall-failed = ошибка    { $what }: { $why }
cli-paths-config = конфиг
cli-paths-ca = CA
cli-paths-plugins = плагины
cli-paths-language = язык
lang-source-flag = флаг --lang
lang-source-config = config.toml
lang-source-os = язык системы
lang-source-default = по умолчанию

# CLI: --help и подкоманды
cmd-about = Плагины в стиле Vencord для Discord через локальный прокси
cmd-run = Запустить прокси (по умолчанию). Направьте Discord на него: `--proxy-server=http://127.0.0.1:18463`
cmd-launch = Запустить прокси и Discord через него (сначала закройте Discord). Можно указать команду клиента
cmd-setup = Первая настройка: создать конфиг и локальный сертификат CA, добавить его в доверенные и включить запуск прокси при входе
cmd-uninstall = Отменить всё, что сделал `setup`: автозапуск, доверие к CA, созданные файлы (ваш конфиг останется)
cmd-trust = Добавить локальный сертификат CA в хранилища, которые использует Discord
cmd-untrust = Убрать локальный сертификат CA из этих хранилищ
cmd-plugins = Показать плагины и их состояние
cmd-enable = Включить плагин
cmd-disable = Выключить плагин
cmd-toggle = Переключить плагин (например, `phoecord toggle game_activity_toggle`)
cmd-log = Показать последние удалённые и изменённые сообщения, сохранённые плагином message_logger
cmd-import-vencord = Включить плагины PhoeCord, которые совпадают с включёнными плагинами Vencord / Vesktop / Equicord
cmd-selftest = Проверить, что всё работает с настоящим Discord (вход в аккаунт не нужен)
cmd-paths = Показать, где лежат конфиг, сертификат CA и данные плагинов
cli-arg-import-path = Путь к settings.json Vencord (если не указан, ищется сам)
cli-import-none = настройки Vencord, Vesktop или Equicord не найдены; укажите файл через --path FILE
cli-import-found = Читаю настройки { $name }: { $path }
cli-import-read = не удалось прочитать { $path }
cli-import-summary = Включено: { $enabled }, уже было включено: { $already }, без аналога в PhoeCord: { $unmatched } плагин(ов) Vencord.
cli-help-usage = Использование:
cli-help-commands = Команды
cli-help-options = Параметры
cli-help-arguments = Аргументы
cli-help-help = Показать справку
cli-help-version = Показать версию
cli-arg-lang = Язык интерфейса: en или ru (по умолчанию: `language` в config.toml, затем язык системы)
cli-arg-log-n = Сколько записей показать (по умолчанию 20)

# CLI: run, launch
cli-listening = PhoeCord слушает на { $listen } (плагины: { $plugins })
cli-starting = запускаю { $program }
cli-launcher-exited = лаунчер завершился ({ $status }); прокси продолжает работать, остановить: Ctrl+C
cli-client-exited = Discord завершился ({ $status })
cli-err-listen = не удалось слушать на { $listen }
cli-err-start-discord = не удалось запустить Discord
cli-err-no-discord-linux = Discord не найден в PATH; укажите команду: phoecord launch -- /путь/к/discord
cli-err-no-discord-windows = Discord не найден в %LOCALAPPDATA%; укажите команду: phoecord launch -- C:\путь\к\Discord.exe
cli-err-no-discord-macos = Discord не найден в /Applications; укажите команду: phoecord launch -- /путь/к/Discord
cli-error-prefix = ошибка
cli-warning-prefix = предупреждение

# CLI: plugins, enable, disable, toggle
cli-plugin-on = вкл
cli-plugin-off = выкл
cli-plugins-vencord = (в Vencord: { $name })
cli-no-such-plugin = плагина «{ $name }» нет (список: `phoecord plugins`)
cli-plugin-state-on = { $name }: включён
cli-plugin-state-off = { $name }: выключен
cli-gateway-note = (плагин шлюза: заработает после переподключения Discord; его вызывает Ctrl+R в Discord)
cli-config-line = конфиг: { $path }
cli-ca-line = локальный сертификат CA: { $path } (действует только для { $hosts })
plugin-desc-fake_nitro = Отправляйте любые эмодзи без Nitro (ссылками на картинки)
plugin-desc-game_activity_toggle = Пока включён, в вашем статусе не видно игру, в которую вы играете
plugin-desc-message_logger = Сохраняет удалённые сообщения и показывает историю правок
plugin-desc-no_track = Блокирует аналитику и отчёты о сбоях Discord
plugin-desc-relationship_notifier = Сообщает, когда вас убрали из друзей, с сервера или из группы
plugin-desc-silent_typing = Индикатор набора текста никогда не отправляется
plugin-desc-clear_urls = Убирает параметры отслеживания из ссылок, которые вы отправляете
plugin-desc-commands = Слэш-команда /phoecord: plugins, toggle
plugin-desc-i_love_spam = Не скрывает сообщения «вероятных спамеров»
plugin-desc-no_blocked_messages = Полностью скрывает сообщения заблокированных и игнорируемых пользователей
plugin-desc-no_reply_mention = По умолчанию отключает уведомление автору при ответе
plugin-desc-text_replace = Заменяет текст в отправляемых сообщениях по вашим правилам
plugin-desc-send_timestamps = Превращает `12:34` в сообщениях в метку времени Discord
plugin-desc-anonymise_file_names = Заменяет имена загружаемых файлов на случайные, расширение сохраняется
plugin-desc-silent_message_toggle = Отправляет сообщения без уведомлений

# CLI: log
cli-log-empty = пока ничего не записано ({ $path })
cli-log-place-channel = в канале { $channel }
cli-log-place-guild = на сервере { $guild }, в канале { $channel }
cli-log-edited = { $time }  ✏ { $author } { $place }: { $was } → { $content }{ $tail }
cli-log-deleted = { $time }  🗑 { $author } { $place }: { $content }{ $tail }

# CLI: отчёт удаления (что убрано)
cli-step-dir = папка { $path }
cli-step-file = файл { $path }
cli-step-ca-trusted = сертификат CA { $path } из системных хранилищ доверенных сертификатов
cli-step-run-key = значение Run в реестре { $name }
cli-step-launch-agent = LaunchAgent { $label }
cli-step-systemd-unit = пользовательская служба systemd { $unit }
cli-kept-already-gone = уже удалено
cli-kept-not-empty = не пуста
cli-kept-holds-log = в ней журнал неудавшихся шагов
cli-kept-line = оставлено { $what } ({ $why })

# CLI: selftest
cli-selftest-rest-ok = ✓ REST: discord.com перехвачен, локальный сертификат CA принят, API доступен
cli-selftest-rest-fail = ✗ REST: { $reason }
cli-selftest-rest-timeout = ✗ REST: время ожидания истекло
cli-selftest-gateway-ok = ✓ шлюз { $combo }: HELLO прошёл туда и обратно (heartbeat { $ms } мс)
cli-selftest-gateway-fail = ✗ шлюз { $combo }: { $reason }
cli-selftest-gateway-timeout = ✗ шлюз { $combo }: время ожидания истекло
cli-selftest-undecodable = ✗ PhoeCord не смог разобрать пакетов шлюза: { $count }
cli-selftest-decoded = прокси разобрал пакетов: { $count }
cli-selftest-failed = не прошло проверок: { $count }
cli-selftest-passed = все проверки пройдены
cli-selftest-err-connect-closed = прокси закрыл соединение CONNECT
cli-selftest-err-proxy-answered = прокси ответил: { $line }
cli-selftest-err-http = HTTP { $status }: { $body }
cli-selftest-err-handshake = рукопожатие
cli-selftest-err-closed-before-hello = соединение закрыто до HELLO
cli-selftest-err-expected-hello = ожидался HELLO, получен op { $op }

# Условия использования (TERMS.ru.md)
terms-prompt = Введите `да` (или `yes`), чтобы принять условия и установить; всё остальное отменяет:
terms-declined = Условия не приняты; ничего не установлено.
cli-arg-accept-terms = Не спрашивать про условия: вы прочитали TERMS.ru.md и принимаете их

# Исправления 1.4.8
cli-uninstall-stopped = остановлен прокси PhoeCord (процесс { $pid })
cli-uninstall-stop-failed = не удалось остановить работающий прокси PhoeCord: { $why }
cli-arg-channel = Какой Discord запустить: stable, ptb или canary (нужно, если установлено несколько)
cli-err-channel-not-installed = Discord { $channel } не установлен
cli-err-several-clients = установлено несколько клиентов Discord, поэтому ни один не выбран за вас. Укажите --channel:
    { $list }
cli-proxy-reused = Прокси PhoeCord (версия { $version }) уже работает на { $listen }; запускаю Discord через него.
cli-err-port-taken = Адрес { $listen } занят другой программой, не PhoeCord. Закройте её или измените `listen` в config.toml.
cli-arg-setup-channel = Добавить ярлык в меню «Пуск», который запускает этот Discord через PhoeCord: stable, ptb или canary (по умолчанию: единственный установленный)
cli-shortcut-created = ярлык «{ $name }» добавлен: { $path }
cli-shortcut-unsupported = В этой системе ярлык не создаётся; запускайте Discord командой `phoecord launch --channel { $channel }`.
cli-shortcut-skipped = Ярлык не добавлен: { $why }
