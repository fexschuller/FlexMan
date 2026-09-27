import * as backend from "../backend";
import { errorMessage } from "../models";
import { useToast } from "./ToastProvider";

interface PathLinkProps {
  path: string;
  target: backend.RevealTarget;
}

// A real <button role="link"> (Pico styles it like an <a>) rather than an
// <a href="#"> - the latter looks like a link but isn't keyboard/AT friendly
// since it doesn't actually navigate anywhere.
function PathLink({ path, target }: PathLinkProps) {
  const showToast = useToast();

  async function handleClick() {
    try {
      await backend.revealPath(target);
    } catch (err) {
      showToast(errorMessage(err));
    }
  }

  return (
    <button type="button" role="link" onClick={handleClick}>
      {path}
    </button>
  );
}

export default PathLink;
