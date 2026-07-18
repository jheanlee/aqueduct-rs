/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import { useEffect, useState } from "react";
import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { editUserSchema } from "@/form-schemas/users/edit-user.ts";
import { z } from "zod";
import { deleteTunnelUser, editTunnelUser } from "@/services/tunnel/users.ts";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog.tsx";
import { Button } from "@/components/ui/button.tsx";
import { Pencil } from "lucide-react";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldSet,
} from "@/components/ui/field.tsx";
import { Input } from "@/components/ui/input.tsx";
import { Checkbox } from "@/components/ui/checkbox.tsx";
import { Separator } from "@/components/ui/separator.tsx";
import { toast } from "sonner";

interface UserEntry {
  id: string;
  username: string;
  label: string[];
  administrator: boolean;
}

interface EditUserInterface {
  onClose: () => void;
  user: UserEntry;
}

export const EditUser = ({ onClose, user }: EditUserInterface) => {
  //  changable: set-new-password label admin
  const [submitStatus, setSubmitStatus] = useState<number>(200);
  const [dialogOpen, setDialogOpen] = useState<boolean>(false);
  const [showPasswordInput, setShowPasswordInput] = useState<boolean>(false);

  const getSubmitStatusMessage = (status: number) => {
    switch (status) {
      case 404:
        return "This user does not exist.";
      case 500:
        return "Unable to connect to the server.";
      default:
        return `An error has occurred. Error code: ${status}`;
    }
  };

  const form = useForm<z.infer<typeof editUserSchema>>({
    resolver: zodResolver(editUserSchema),
    defaultValues: {
      password: null,
      label: user.label,
      administrator: user.administrator,
    },
  });

  const onSubmit = async (values: z.infer<typeof editUserSchema>) => {
    const res = await editTunnelUser(user.id, values);
    setSubmitStatus(res);

    if (res === 200) {
      setDialogOpen(false);
      toast.success(`Successfully updated user "${user.username}"`);
      onClose();
    }
  };

  useEffect(() => {
    if (!dialogOpen) {
      form.reset();
      setSubmitStatus(200);
      setShowPasswordInput(false);
    }
  }, [dialogOpen]);

  return (
    <Dialog open={dialogOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" onClick={() => setDialogOpen(true)}>
          <Pencil />
        </Button>
      </DialogTrigger>
      <div>
        <DialogContent>
          <form
            onSubmit={form.handleSubmit(onSubmit)}
            className="flex flex-col gap-4"
          >
            <DialogHeader>
              <DialogTitle>{`Edit User "${user.username}"`}</DialogTitle>
            </DialogHeader>
            <FieldSet
              data-invalid={submitStatus !== 200}
              className="max-h-[50vh] overflow-y-auto"
            >
              <FieldGroup>
                <Field>
                  <FieldLabel>Username</FieldLabel>
                  <Input type="text" value={user.username} disabled />
                </Field>
                <Controller
                  name="password"
                  control={form.control}
                  render={({ field, fieldState }) => (
                    <Field>
                      <FieldLabel>Password</FieldLabel>
                      {!showPasswordInput && (
                        <Button
                          type="button"
                          variant="outline"
                          onClick={() => setShowPasswordInput(true)}
                        >
                          <Pencil />
                          Change
                        </Button>
                      )}
                      {showPasswordInput && (
                        <Input
                          type="password"
                          placeholder="••••••••"
                          aria-invalid={fieldState.invalid}
                          {...field}
                        />
                      )}
                      {fieldState.invalid && (
                        <FieldError errors={[fieldState.error]} />
                      )}
                    </Field>
                  )}
                />
                <Controller
                  name="label"
                  control={form.control}
                  render={({ field, fieldState }) => (
                    <Field>
                      <FieldLabel>Labels</FieldLabel>
                      <Input
                        type="text"
                        placeholder="label1,label2"
                        value={
                          field.value === undefined ? "" : field.value.join(",")
                        }
                        aria-invalid={fieldState.invalid}
                        onChange={(event) => {
                          const value = event.target.value.split(",");
                          form.setValue(
                            "label",
                            value.filter((str) => str.length > 0),
                          );
                        }}
                      />
                      <FieldDescription>
                        Create multiple labels by separating them with a single
                        comma (,)
                      </FieldDescription>
                      {fieldState.invalid && (
                        <FieldError errors={[fieldState.error]} />
                      )}
                    </Field>
                  )}
                />
                <Controller
                  name="administrator"
                  control={form.control}
                  render={({ field, fieldState }) => (
                    <Field>
                      <div className="flex flex-row gap-1 items-center">
                        <Checkbox
                          checked={field.value}
                          aria-invalid={fieldState.invalid}
                          onCheckedChange={field.onChange}
                        />
                        <FieldLabel>Administrator</FieldLabel>
                      </div>
                      {fieldState.invalid && (
                        <FieldError errors={[fieldState.error]} />
                      )}
                    </Field>
                  )}
                />
                <Separator />
                <Field>
                  <Button
                    type="button"
                    variant="outline"
                    className="text-destructive"
                    onClick={() => {
                      const deleteUser = async () => {
                        const res = await deleteTunnelUser(user.id);
                        if (res === 200) {
                          toast.success(
                            `Successfully deleted user "${user.username}"`,
                          );
                        } else {
                          toast.error(getSubmitStatusMessage(res));
                        }
                        onClose();
                      };

                      void deleteUser();
                      setDialogOpen(false);
                    }}
                  >
                    Delete User
                  </Button>
                </Field>
                {submitStatus !== 200 && (
                  <FieldError>
                    {getSubmitStatusMessage(submitStatus)}
                  </FieldError>
                )}
              </FieldGroup>
            </FieldSet>
            <DialogFooter className="grid grid-cols-2">
              <DialogClose onClick={() => setDialogOpen(false)} asChild>
                <Button variant="outline">Cancel</Button>
              </DialogClose>
              <Field>
                <Button type="submit">Update</Button>
              </Field>
            </DialogFooter>
          </form>
        </DialogContent>
      </div>
    </Dialog>
  );
};
