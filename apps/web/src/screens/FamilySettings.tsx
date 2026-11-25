import { useCallback, useEffect, useState, type FormEvent } from "react";
import { Avatar, Button, KID_COLOURS, SelectField, Sheet, TextField } from "@kids-economy/design-system";
import { api, ApiFailure, type FamilyView, type KidSummary, type Role } from "../api";
import { shareableUrl } from "../router";
import { shortDay } from "../words";
import { BackToFamily, CopyLink, ErrorText } from "./parts";
import { Screen } from "./Screen";

export const ROLE_WORD: Record<Role, string> = { owner: "Owner", parent: "Parent", viewer: "Viewer" };

type Member = FamilyView["members"][number];

/**
 * `/families/{id}/settings`: the Family's Kids and adults. Owners and Parents fix, hide and
 * remove Kids; Owners invite and reset passwords.
 */
export function FamilySettings({ familyId }: { familyId: string }) {
  const [family, setFamily] = useState<FamilyView | null>(null);
  const [inviting, setInviting] = useState(false);
  const [resetting, setResetting] = useState<Member | null>(null);
  const [editing, setEditing] = useState<KidSummary | null>(null);

  const reload = useCallback(() => {
    void api.family(familyId).then(setFamily);
  }, [familyId]);
  useEffect(reload, [reload]);

  if (!family) return <p className="p-4 text-muted">Loading…</p>;
  const isOwner = family.you.role === "owner";
  const mayManageKids = family.you.role !== "viewer";

  return (
    <Screen>
      <header className="flex items-center">
        <BackToFamily />
      </header>
      <h1 className="m-0 font-display text-[28px] leading-[34px] font-semibold">Family settings</h1>

      {family.kids.length > 0 && (
        <section className="flex flex-col gap-2">
          <h2 id="kids" className="m-0 font-display text-[22px] leading-7 font-semibold">
            Kids
          </h2>
          <ul aria-labelledby="kids" className="m-0 flex list-none flex-col p-0">
            {family.kids.map((kid, index) => (
              <li key={kid.id} className="flex items-center gap-3 border-b border-line py-3 last:border-b-0">
                <Avatar name={kid.name} colour={KID_COLOURS[index % KID_COLOURS.length]} />
                <span className="min-w-0 grow">
                  <span className="block text-[19px] leading-[26px] font-bold [overflow-wrap:anywhere]">{kid.name}</span>
                  {kid.hidden && (
                    <span className="block text-sm font-semibold text-muted">Hidden from Family home</span>
                  )}
                </span>
                {mayManageKids && (
                  <Button
                    variant="quiet"
                    className="flex-none px-3"
                    aria-label={`Edit ${kid.name}`}
                    onClick={() => setEditing(kid)}
                  >
                    Edit
                  </Button>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}

      <section className="flex flex-col gap-2">
        <h2 id="members" className="m-0 font-display text-[22px] leading-7 font-semibold">
          Members
        </h2>
        <ul aria-labelledby="members" className="m-0 flex list-none flex-col p-0">
          {family.members.map((member) => (
            <li key={member.id} className="flex items-center gap-3 border-b border-line py-3 last:border-b-0">
              <span className="min-w-0 grow">
                <span className="block text-[19px] leading-[26px] font-bold">
                  {member.name}
                  {member.id === family.you.member_id && <span className="font-semibold text-muted"> (you)</span>}
                </span>
                <span className="block text-sm font-semibold text-muted [overflow-wrap:anywhere]">
                  {ROLE_WORD[member.role]} · {member.email}
                </span>
              </span>
              {isOwner && member.id !== family.you.member_id && (
                <Button
                  variant="quiet"
                  className="flex-none px-3"
                  aria-label={`Reset password for ${member.name}`}
                  onClick={() => setResetting(member)}
                >
                  Reset password
                </Button>
              )}
            </li>
          ))}
        </ul>
      </section>

      {family.invitations.length > 0 && (
        <section className="flex flex-col gap-2">
          <h2 id="waiting" className="m-0 font-display text-[22px] leading-7 font-semibold">
            Waiting to join
          </h2>
          <ul aria-labelledby="waiting" className="m-0 flex list-none flex-col p-0">
            {family.invitations.map((invitation) => (
              <li key={invitation.id} className="border-b border-line py-3 last:border-b-0">
                <span className="block text-[17px] font-bold [overflow-wrap:anywhere]">{invitation.email}</span>
                <span className="text-sm font-semibold text-muted">
                  {ROLE_WORD[invitation.role]} · link works until {shortDay(invitation.expires_at.slice(0, 10))}
                </span>
              </li>
            ))}
          </ul>
        </section>
      )}

      {isOwner && (
        <Button variant="secondary" size="lg" onClick={() => setInviting(true)}>
          Invite an adult
        </Button>
      )}

      {inviting && (
        <InviteSheet
          familyId={familyId}
          onClose={() => {
            setInviting(false);
            reload();
          }}
        />
      )}
      {editing && (
        <KidSheet
          familyId={familyId}
          kid={editing}
          onClose={() => setEditing(null)}
          onChanged={() => {
            setEditing(null);
            reload();
          }}
        />
      )}
      {resetting && <ResetSheet familyId={familyId} member={resetting} onClose={() => setResetting(null)} />}
    </Screen>
  );
}

const INVITE_ERRORS: Record<string, string> = {
  invalid_email: "That does not look like an email address.",
  already_member: "This adult is already in the Family.",
  invitation_pending: "This email already has a link waiting. It is listed under Waiting to join.",
};

function InviteSheet({ familyId, onClose }: { familyId: string; onClose: () => void }) {
  const [email, setEmail] = useState("");
  const [role, setRole] = useState<Role>("viewer");
  const [link, setLink] = useState<string | null>(null);
  const [error, setError] = useState<string | undefined>();

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setError(undefined);
    try {
      const made = await api.invite(familyId, email.trim(), role);
      setLink(shareableUrl(made.link));
    } catch (failure) {
      setError((failure instanceof ApiFailure && INVITE_ERRORS[failure.code]) || "Could not invite right now.");
    }
  };

  return (
    <Sheet
      open
      title="Invite an adult"
      description="A second Parent, a grandparent, an aunt. You get a link to send them."
      onClose={onClose}
      actions={
        link ? undefined : (
          <Button type="submit" form="invite" size="lg" disabled={email.trim() === ""}>
            Make Invitation link
          </Button>
        )
      }
    >
      {link ? (
        <>
          <CopyLink label="Invitation link" url={link} />
          <p className="m-0 text-[15px] leading-5 font-semibold text-muted">
            Send it any way you like. It works for 7 days, only for {email.trim()}. You see it only now.
          </p>
        </>
      ) : (
        <form id="invite" onSubmit={submit} className="flex flex-col gap-4">
          <TextField label="Their email" type="email" autoComplete="off" value={email} onChange={setEmail} required />
          <SelectField
            label="Role"
            value={role}
            onChange={(value) => setRole(value as Role)}
            options={[
              { value: "viewer", label: "Viewer: sees the Kids' money" },
              { value: "parent", label: "Parent: also records money" },
              { value: "owner", label: "Owner: also invites adults" },
            ]}
          />
          {error && <ErrorText>{error}</ErrorText>}
        </form>
      )}
    </Sheet>
  );
}

function ResetSheet({ familyId, member, onClose }: { familyId: string; member: Member; onClose: () => void }) {
  const [link, setLink] = useState<string | null>(null);
  const [error, setError] = useState<string | undefined>();
  const make = async () => {
    setError(undefined);
    try {
      const made = await api.passwordResetLink(familyId, member.id);
      setLink(shareableUrl(made.link));
    } catch {
      setError("Could not make a reset link right now.");
    }
  };
  return (
    <Sheet
      open
      title={`Reset ${member.name}'s password`}
      description={`You get a link to send to ${member.name}. Opening it lets them choose a new password and signs them out everywhere else.`}
      onClose={onClose}
      actions={
        link ? undefined : (
          <Button size="lg" onClick={() => void make()}>
            Make reset link
          </Button>
        )
      }
    >
      {link && (
        <>
          <CopyLink label="Password reset link" url={link} />
          <p className="m-0 text-[15px] leading-5 font-semibold text-muted">
            Works once, for 24 hours. Send it only to {member.name}. You see it only now.
          </p>
        </>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </Sheet>
  );
}

/** Same limit as the API's KidName. */
const KID_NAME_MAX = 40;

/** Fixes a Kid's name, hides or shows them on the Family home, or removes them. */
function KidSheet({
  familyId,
  kid,
  onClose,
  onChanged,
}: {
  familyId: string;
  kid: KidSummary;
  onClose: () => void;
  onChanged: () => void;
}) {
  const [name, setName] = useState(kid.name);
  const [asking, setAsking] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const trimmed = name.trim();
  const letters = [...trimmed].length;
  const nameProblem = letters > KID_NAME_MAX ? `Use at most ${KID_NAME_MAX} letters (this has ${letters}).` : undefined;
  const nameReady = trimmed !== "" && !nameProblem && trimmed !== kid.name;

  const run = async (action: () => Promise<unknown>) => {
    setBusy(true);
    setError(undefined);
    try {
      await action();
      onChanged();
    } catch {
      setError("Could not save right now. Try again.");
    } finally {
      setBusy(false);
    }
  };
  const rename = (event: FormEvent) => {
    event.preventDefault();
    if (nameReady) void run(() => api.changeKid(familyId, kid.id, { name: trimmed }));
  };

  if (asking) {
    return (
      <Sheet
        open
        title={`Remove ${kid.name}?`}
        description={`${kid.name}'s Wallet and history are kept, but no one in the Family sees them after this. To keep them and only tidy the Family home, hide ${kid.name} instead.`}
        onClose={() => setAsking(false)}
        actions={
          <Button
            variant="danger"
            size="lg"
            disabled={busy}
            onClick={() => void run(() => api.removeKid(familyId, kid.id))}
          >
            Yes, remove {kid.name}
          </Button>
        }
      >
        {error && <ErrorText>{error}</ErrorText>}
      </Sheet>
    );
  }

  return (
    <Sheet open title={`Edit ${kid.name}`} onClose={onClose}>
      <form onSubmit={rename} className="flex flex-col gap-3">
        <TextField
          label="Name"
          hint="Fix a name typed wrong. Their money stays the same."
          value={name}
          onChange={setName}
          error={nameProblem}
          autoComplete="off"
        />
        <Button type="submit" variant="secondary" disabled={!nameReady || busy}>
          Save name
        </Button>
      </form>
      <div className="flex flex-col gap-2 border-t border-line pt-4">
        <p className="m-0 text-[15px] leading-5 font-semibold text-muted">
          {kid.hidden
            ? `${kid.name} is hidden from the Family home.`
            : `Hiding takes ${kid.name} off the Family home. Their money stays and keeps working.`}
        </p>
        <Button
          variant="secondary"
          disabled={busy}
          onClick={() => void run(() => api.changeKid(familyId, kid.id, { hidden: !kid.hidden }))}
        >
          {kid.hidden ? "Show on Family home" : "Hide from Family home"}
        </Button>
      </div>
      <div className="flex flex-col gap-2 border-t border-line pt-4">
        <Button variant="danger" disabled={busy} onClick={() => setAsking(true)}>
          Remove {kid.name}
        </Button>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </Sheet>
  );
}
