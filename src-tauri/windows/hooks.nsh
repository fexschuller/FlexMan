; FlexASIO's own driver (control_panel.cpp) looks up this exact registry
; key/value when a DAW's "ASIO Control Panel" button is clicked, and
; launches "<InstallPath>\FlexASIOGUI.exe" if found. Registering it here
; is what makes that button open this app instead of doing nothing.
;
; The NSIS installer is a 32-bit process, so without SetRegView the write
; would be redirected to WOW6432Node. The 64-bit driver reads the 64-bit
; view and the 32-bit driver the 32-bit view, so write to both.
;
; The key is shared with the original FlexASIOGUI (C#). On uninstall only
; remove it if it still points at us, so we don't break the other app.

!macro NSIS_HOOK_POSTINSTALL
  SetRegView 64
  WriteRegStr HKLM "Software\Fabrikat\FlexASIOGUI\Install" "InstallPath" "$INSTDIR"
  SetRegView 32
  WriteRegStr HKLM "Software\Fabrikat\FlexASIOGUI\Install" "InstallPath" "$INSTDIR"
  SetRegView lastused
!macroend

!macro FLEXMAN_REMOVE_INSTALL_KEY
  ReadRegStr $0 HKLM "Software\Fabrikat\FlexASIOGUI\Install" "InstallPath"
  ${If} $0 == "$INSTDIR"
    DeleteRegKey HKLM "Software\Fabrikat\FlexASIOGUI\Install"
    DeleteRegKey /ifempty HKLM "Software\Fabrikat\FlexASIOGUI"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  SetRegView 64
  !insertmacro FLEXMAN_REMOVE_INSTALL_KEY
  SetRegView 32
  !insertmacro FLEXMAN_REMOVE_INSTALL_KEY
  SetRegView lastused
!macroend
