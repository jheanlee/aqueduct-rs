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

import { Controller, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { createUserSchema } from "@/form-schemas/users/create-user.ts";
import { z } from "zod";
import {
  Field,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldSet,
} from "@/components/ui/field.tsx";
import { useState } from "react";
import { Input } from "@/components/ui/input.tsx";
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
import { Plus } from "lucide-react";
import { Checkbox } from "@/components/ui/checkbox.tsx";
import { createTunnelUser } from "@/services/tunnel/users.ts";

interface CreateUserInterface {
  onClose: () => void;
}

export const CreateUser = ({ onClose }: CreateUserInterface) => {
  const [submitStatus, setSubmitStatus] = useState<number>(200);
  const [dialogOpen, setDialogOpen] = useState<boolean>(false);

  const getSubmitStatusMessage = () => {
    switch (submitStatus) {
      case 409:
        return "A user with this username already exist.";
      case 500:
        return "Unable to connect to the server.";
      default:
        return `An error has occurred. Error code: ${submitStatus}`;
    }
  };

  const form = useForm<z.infer<typeof createUserSchema>>({
    resolver: zodResolver(createUserSchema),
    defaultValues: {
      username: "",
      password: "",
      label: [],
      administrator: false,
    },
  });

  const onSubmit = async (values: z.infer<typeof createUserSchema>) => {
    const res = await createTunnelUser(values);
    setSubmitStatus(res);

    if (res === 200) {
      setDialogOpen(false);
      onClose();
    }
  };

  return (
    <Dialog open={dialogOpen}>
      <DialogTrigger asChild>
        <Button
          variant="outline"
          className="w-32"
          onClick={() => setDialogOpen(true)}
        >
          <Plus />
          Create User
        </Button>
      </DialogTrigger>
      <div>
        <DialogContent>
          <form
            onSubmit={form.handleSubmit(onSubmit)}
            className="flex flex-col gap-4"
          >
            <DialogHeader>
              <DialogTitle>Create User</DialogTitle>
            </DialogHeader>
            <FieldSet
              data-invalid={submitStatus !== 200}
              className="max-h-[50vh] overflow-y-auto"
            >
              <FieldGroup>
                <Controller
                  name="username"
                  control={form.control}
                  render={({ field, fieldState }) => (
                    <Field>
                      <FieldLabel>Username</FieldLabel>
                      <Input
                        type="text"
                        placeholder="user"
                        aria-invalid={fieldState.invalid}
                        {...field}
                      />
                      {fieldState.invalid && (
                        <FieldError errors={[fieldState.error]} />
                      )}
                    </Field>
                  )}
                />
                <Controller
                  name="password"
                  control={form.control}
                  render={({ field, fieldState }) => (
                    <Field>
                      <FieldLabel>Password</FieldLabel>
                      <Input
                        type="password"
                        placeholder="••••••••"
                        aria-invalid={fieldState.invalid}
                        {...field}
                      />
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
                          form.setValue("label", event.target.value.split(","));
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
                {submitStatus !== 200 && (
                  <FieldError>{getSubmitStatusMessage()}</FieldError>
                )}
              </FieldGroup>
            </FieldSet>
            <DialogFooter className="grid grid-cols-2">
              <DialogClose onClick={() => setDialogOpen(false)} asChild>
                <Button variant="outline">Cancel</Button>
              </DialogClose>
              <Field>
                <Button type="submit">Create</Button>
              </Field>
            </DialogFooter>
          </form>
        </DialogContent>
      </div>
    </Dialog>
  );
};
