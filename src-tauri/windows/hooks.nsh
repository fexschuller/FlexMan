; FlexASIO's own driver (control_panel.cpp) looks up this exact registry
; key/value when a DAW's "ASIO Control Panel" button is clicked, and
; launches "<InstallPath>\FlexASIOGUI.exe" if found. Registering it here
; is what makes that button open this app instead of doing nothing.

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKLM "Software\Fabrikat\FlexASIOGUI\Install" "InstallPath" "$INSTDIR"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegKey HKLM "Software\Fabrikat\FlexASIOGUI\Install"
  DeleteRegKey /ifempty HKLM "Software\Fabrikat\FlexASIOGUI"
!macroend
