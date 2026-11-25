import { useCallback, useEffect, useState, type FormEvent } from "react";
import { Button, SelectField, Sheet, TextField } from "@kids-economy/design-system";
import { api, ApiFailure, type FamilyView, type Role } from "../api";
import { shareableUrl } from "../router";
import { shortDay } from "../words";
import { BackToFamily, CopyLink, ErrorText } from "./parts";
import { Screen } from "./Screen";

export const ROLE_WORD: Record<Role, string> = { owner: "Owner", parent: "Parent", viewer: "Viewer" };

type Member = FamilyView["members"][number];

/** `/families/{id}/members`: the adults in the Family; Owners invite and reset passwords. */
export function FamilyMembers({ familyId }: { familyId: string }) {
  const [family, setFamily] = useState<FamilyView | null>(null);
  const [inviting, setInviting] = useState(false);
  const [resetting, setResetting] = useState<Member | null>(null);

  const reload = useCallback(() => {
    void api.family(familyId).then(setFamily);
  }, [familyId]);
  useEffect(reload, [reload]);

  if (!family) return <p className="p-4 text-muted">Loading…</p>;
  const isOwner = family.you.role === "owner";

  return (
    <Screen>
      <header className="flex items-center">
        <BackToFamily />
      </header>
      <h1 className="m-0 font-display text-[28px] leading-[34px] font-semibold">Family members</h1>

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
